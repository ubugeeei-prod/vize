pub(super) const TRACE_STDIO_PROXY: &str = r#"use strict;
use warnings;
use IO::Select;
use IO::Socket::INET;
use IPC::Open3;
use FindBin;
require "$FindBin::Bin/native-proxy-lifecycle.pl";

my (
    $client_trace_path,
    $server_trace_path,
    $stderr_path,
    $gate_sentinel,
    $gate_port,
    $actual,
    @args,
) = @ARGV;
open my $client_trace, '>:raw', $client_trace_path or die "open client trace: $!";
open my $server_trace, '>:raw', $server_trace_path or die "open server trace: $!";
open my $server_stderr, '>:raw', $stderr_path or die "open server stderr: $!";
binmode STDIN;
binmode STDOUT;

my $lifecycle = NativeProxyLifecycle->new($actual, "$FindBin::Bin/native-proxy-initialize.json");
my $client_fd = fileno(STDIN);
my $native_input_closed = 0;
my $client_reader_closed = 0;
$lifecycle->{capturePointer} = "$FindBin::Bin/native-proxy-capture.path";
$lifecycle->{tracePaths} = { client => $client_trace_path, server => $server_trace_path,
    stderr => $stderr_path };
my ($server_stdin, $server_stdout);
my $server_pid = open3($server_stdin, $server_stdout, ">&" . fileno($server_stderr), $actual, @args);
binmode $server_stdin;
binmode $server_stdout;
close $server_stderr;

my $ready = IO::Select->new();
$ready->add(*STDIN);
$ready->add($server_stdout);
my $tail = '';
my $shutdown_seen = 0;

sub write_all {
    my ($handle, $bytes) = @_;
    while (length $bytes) {
        my $written = syswrite $handle, $bytes;
        die "write proxy stream: $!" unless defined $written;
        substr($bytes, 0, $written, '');
    }
}

while ($ready->count) {
    for my $handle ($ready->can_read) {
        my $fd = fileno($handle);
        next unless defined $fd;
        next if $client_reader_closed && $fd == $client_fd;
        my $read = sysread $handle, my $chunk, 8192;
        die "read proxy stream: $!" unless defined $read;
        if ($read == 0) {
            $ready->remove($handle);
            if ($fd == $client_fd) {
                $lifecycle->eof('client');
            } else {
                $lifecycle->eof('server');
                $ready->remove(*STDIN);
            }
            close $server_stdin unless $native_input_closed;
            $native_input_closed = 1;
            next;
        }
        if ($fd == $client_fd) {
            write_all($client_trace, $chunk);
            for my $frame ($lifecycle->frames('client', $chunk)) {
                my ($raw, $message) = @$frame;
                if ($lifecycle->client($raw, $message, $server_pid)) {
                    close $server_stdin or die "close authenticated native stdin: $!";
                    $native_input_closed = 1;
                    $lifecycle->{nativeInputEofAfterExit} = 1;
                    $lifecycle->retire_client_reader();
                    $ready->remove(*STDIN);
                    close STDIN or die "close client reader after authenticated Exit: $!";
                    # Keep descriptor zero read-only while the final receipt is opened.
                    open STDIN, '<:raw', '/dev/null' or die "reserve retired client descriptor: $!";
                    $client_reader_closed = 1;
                    $lifecycle->{clientReaderClosedAfterExit} = 1;
                } elsif (!$native_input_closed) {
                    write_all($server_stdin, $raw);
                } else {
                    $lifecycle->error('client bytes after native stdin closed', $raw);
                }
            }
            $tail .= $chunk;
            if (!$shutdown_seen && $tail =~ /"method"\s*:\s*"shutdown"/) {
                $shutdown_seen = 1;
                if (-e $gate_sentinel) {
                    my $gate = IO::Socket::INET->new(
                        PeerAddr => '127.0.0.1',
                        PeerPort => $gate_port,
                        Proto => 'tcp',
                    ) or die "connect shutdown gate: $!";
                    write_all($gate, 'S');
                    my $gate_read = sysread $gate, my $release, 1;
                    die "read shutdown gate release: $!" unless defined $gate_read;
                    die "invalid shutdown gate release"
                        unless $gate_read == 1 && $release eq 'R';
                }
            }
            $tail = substr($tail, -128) if length($tail) > 128;
        } else {
            write_all($server_trace, $chunk);
            write_all(*STDOUT, $chunk);
            for my $frame ($lifecycle->frames('server', $chunk)) {
                $lifecycle->server($frame->[1], $server_pid);
            }
        }
    }
}

my $reaped_pid = waitpid($server_pid, 0);
my $status = $?;
my $refused = $lifecycle->finish($server_pid, $status, "$stderr_path.lifecycle.json", $reaped_pid);
exit $refused if $status == 0 && $refused;
exit 1 if $status == -1;
exit 128 + ($status & 127) if $status & 127;
exit $status >> 8;
"#;
