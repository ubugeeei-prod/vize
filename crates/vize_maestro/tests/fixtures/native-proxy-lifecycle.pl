package NativeProxyLifecycle;
use strict;
use warnings;
use Digest::SHA;
use File::Copy qw(copy);
use File::Path qw(make_path);
use JSON::PP;
use Time::HiRes ();

my %KNOWN = map { $_ => 1 } (
    '4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4',
    'a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2',
);
my $CONTRACT = 'a8641e61d1469332cccbd06eb2cbc7a520c17485c53abaefb2e965676902c183';
my $JSON = JSON::PP->new->utf8->canonical;

sub file_identity {
    my ($path) = @_;
    open my $file, '<:raw', $path or return;
    my @before = Time::HiRes::stat($file);
    return unless @before && ($before[2] & 0170000) == 0100000;
    my $digest = Digest::SHA->new(256)->addfile($file)->hexdigest;
    my @after = Time::HiRes::stat($file);
    my @named = Time::HiRes::stat($path);
    my @fields = (0, 1, 2, 7, 9, 10);
    return unless @after && @named;
    for my $index (@fields) {
        return unless $before[$index] == $after[$index] && $before[$index] == $named[$index];
    }
    return { sha256 => $digest, stat => [@before[@fields]] };
}

sub new {
    my ($class, $actual, $contract_path) = @_;
    my $identity = file_identity($actual);
    my $candidate = $identity && $KNOWN{$identity->{sha256}};
    my $expected;
    if ($candidate) {
        open my $file, '<:raw', $contract_path or die "open native initialization contract: $!";
        local $/;
        my $bytes = <$file>;
        die 'changed native initialization contract' unless Digest::SHA::sha256_hex($bytes) eq $CONTRACT;
        $expected = $JSON->decode($bytes);
    }
    return bless { actual => $actual, identity => $identity, candidate => $candidate ? 1 : 0,
        expected => $expected, pending => { client => '', server => '' }, errors => [],
        frames => { client => [], server => [] }, translated => 0 }, $class;
}

sub process_identity {
    my ($self, $pid) = @_;
    my $image = file_identity($self->{actual});
    return unless $image && $JSON->encode($image) eq $JSON->encode($self->{identity});
    return { pid => $pid, image => $image } unless $^O eq 'linux';
    my $stat_path = "/proc/$pid/stat";
    open my $stat_file, '<:raw', $stat_path or return;
    local $/;
    my $raw_stat = <$stat_file>;
    return unless $raw_stat =~ /\A(\d+) \(.*\) (.*)\z/s && $1 == $pid;
    my @fields = split /\s+/, $2;
    return unless @fields >= 20 && $fields[0] =~ /\A[RSDTtI]\z/ && $fields[1] == $$ && $fields[19] > 0;
    my $native = file_identity("/proc/$pid/exe");
    return unless $native && $JSON->encode($native) eq $JSON->encode($image);
    open my $after_file, '<:raw', $stat_path or return;
    my $after = <$after_file>;
    return unless $after =~ /\A(\d+) \(.*\) (.*)\z/s && $1 == $pid;
    my @after = split /\s+/, $2;
    return unless @after >= 20 && $after[0] =~ /\A[RSDTtI]\z/ && $after[1] == $$ && $after[19] == $fields[19];
    return { pid => $pid, parent => $$, birth => $fields[19], image => $image,
        wholeStatBefore => $raw_stat, wholeStatAfter => $after };
}

sub error {
    my ($self, $reason, $raw) = @_;
    push @{$self->{errors}}, { reason => $reason, wholeBytesHex => unpack('H*', $raw) };
}

sub frames {
    my ($self, $side, $bytes) = @_;
    return ([ $bytes, undef ]) unless $self->{candidate};
    $self->{pending}{$side} .= $bytes;
    my @frames;
    while (length $self->{pending}{$side}) {
        my $buffer = $self->{pending}{$side};
        my $end = index($buffer, "\r\n\r\n");
        if ($end < 0) {
            if (length($buffer) > 16 * 1024) {
                $self->error("$side oversized complete header", $buffer);
                $self->{candidate} = 0;
                $self->{pending}{$side} = '';
                push @frames, [ $buffer, undef ];
            }
            last;
        }
        my @lengths = substr($buffer, 0, $end) =~ /^Content-Length:\s*(\d+)\r?$/gim;
        if ($end + 4 > 16 * 1024 || @lengths != 1 || $lengths[0] > 512 * 1024 * 1024) {
            $self->error("$side invalid complete length header", $buffer);
            $self->{candidate} = 0;
            $self->{pending}{$side} = '';
            push @frames, [ $buffer, undef ];
            last;
        }
        my $length = $end + 4 + $lengths[0];
        last if length($buffer) < $length;
        my $raw = substr($self->{pending}{$side}, 0, $length, '');
        my $message = eval { $JSON->decode(substr($raw, $end + 4)); };
        if ($@ || ref($message) ne 'HASH' || ($message->{jsonrpc} // '') ne '2.0') {
            $self->error("$side invalid complete JSON-RPC body", $raw);
            $message = undef;
        }
        push @frames, [ $raw, $message ];
        push @{$self->{frames}{$side}}, { wholeFrameHex => unpack('H*', $raw), message => $message };
    }
    return @frames;
}

sub client {
    my ($self, $raw, $message, $pid) = @_;
    return 0 unless $message;
    if ($self->{translated}) {
        $self->error('complete client frame after translated Exit', $raw);
        return 0;
    }
    if (($message->{method} // '') eq 'initialize' && exists $message->{id}
        && defined $message->{id} && !ref($message->{id})) {
        $self->error('duplicate complete initialize request', $raw) if defined $self->{initializeId};
        $self->{initialized} = 0;
        $self->{initializeId} = $JSON->encode($message->{id});
        $self->{wholeInitializeRequest} = $message;
    }
    if (($message->{method} // '') eq 'shutdown' && exists $message->{id}
        && defined $message->{id} && !ref($message->{id})) {
        $self->error('duplicate complete shutdown request', $raw) if defined $self->{shutdownId};
        $self->{acknowledged} = 0;
        $self->{shutdownId} = $JSON->encode($message->{id});
        $self->{wholeShutdownRequest} = $message;
    }
    return 0 unless ($message->{method} // '') eq 'exit' && !exists $message->{id}
        && !exists $message->{params}
        && !grep { $_ ne 'jsonrpc' && $_ ne 'method' } keys %$message;
    return 0 unless $self->{candidate} && $self->{initialized} && $self->{acknowledged}
        && !@{$self->{errors}};
    my $current = $self->process_identity($pid);
    return 0 unless $current && $current->{pid} == $self->{native}{pid}
        && ($current->{birth} // 0) == ($self->{native}{birth} // 0);
    $self->{wholeExitNotification} = $message;
    $self->{wholeExitFrameHex} = unpack('H*', $raw);
    $self->{nativeBeforeExit} = $current;
    $self->{translated} = 1;
    return 1;
}

sub server {
    my ($self, $message, $pid) = @_;
    return unless $message && exists $message->{id} && !exists $message->{method};
    my $id = $JSON->encode($message->{id});
    if (defined $self->{initializeId} && $id eq $self->{initializeId}) {
        $self->{wholeInitializeResponse} = $message;
        my $native = $self->process_identity($pid);
        $self->{initialized} = !exists $message->{error} && exists $message->{result}
            && $JSON->encode($message->{result}) eq $JSON->encode($self->{expected}) && $native ? 1 : 0;
        $self->{native} = $native if $self->{initialized};
    }
    if (defined $self->{shutdownId} && $id eq $self->{shutdownId}) {
        $self->{wholeShutdownResponse} = $message;
        $self->{acknowledged} = !exists $message->{error} && exists $message->{result}
            && !defined $message->{result} ? 1 : 0;
    }
}

sub eof {
    my ($self, $side) = @_;
    $self->{$side . 'Eof'} = 1;
    if (length $self->{pending}{$side}) {
        $self->error("$side EOF inside an incomplete whole frame", $self->{pending}{$side});
    }
}

sub retire_client_reader {
    my ($self) = @_;
    if (length $self->{pending}{client}) {
        $self->error('partial client frame after translated Exit', $self->{pending}{client});
    }
}

sub finish {
    my ($self, $pid, $status, $path, $reaped_pid) = @_;
    $self->{nativePid} = $pid;
    $self->{nativeWaitStatus} = $status;
    $self->{reapedNativePid} = $reaped_pid;
    $self->error('waitpid did not reap the original native child', '') unless $reaped_pid == $pid;
    if ($self->{translated} && (!($self->{clientEof} || $self->{clientReaderClosedAfterExit}) || !$self->{serverEof}
        || !$self->{nativeInputEofAfterExit})) {
        $self->error('translated Exit without actual native EOF and client reader retirement', '');
    }
    open my $receipt, '>:raw', $path or die "open complete native lifecycle receipt: $!";
    print {$receipt} $JSON->encode({ %$self }) or die "write complete native lifecycle receipt: $!";
    close $receipt or die "close complete native lifecycle receipt: $!";
    if (($ENV{CI} // '') eq 'true' && -f $self->{capturePointer}) {
        open my $pointer, '<:raw', $self->{capturePointer} or die "read owned capture root: $!";
        local $/;
        my $root = <$pointer>;
        die 'invalid owned capture root' if !$root || $root =~ /[\r\n\0]/;
        make_path($root);
        my $capture = "$root/$pid-$$";
        mkdir $capture or die "create new complete native lifecycle capture: $!";
        for my $side (qw(client server stderr)) {
            copy($self->{tracePaths}{$side}, "$capture/$side.raw") or die "retain whole $side: $!";
        }
        copy($path, "$capture/lifecycle.json") or die "retain whole native lifecycle: $!";
    }
    if (@{$self->{errors}}) {
        print STDERR 'Native proxy lifecycle refusal: ', $JSON->encode($self->{errors}), "\n";
        return 1;
    }
    return 0;
}
1;
