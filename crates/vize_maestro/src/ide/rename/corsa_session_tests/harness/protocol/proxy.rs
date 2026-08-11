pub(super) const TRACE_STDIO_PROXY: &str = r#"use strict;
use warnings;
use IO::Select;
use IO::Socket::INET;
use IPC::Open3;

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

my ($server_stdin, $server_stdout);
my $server_pid = open3($server_stdin, $server_stdout, $server_stderr, $actual, @args);
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
        my $read = sysread $handle, my $chunk, 8192;
        die "read proxy stream: $!" unless defined $read;
        if ($read == 0) {
            $ready->remove($handle);
            if (fileno($handle) == fileno(STDIN)) {
                close $server_stdin;
            } else {
                $ready->remove(*STDIN);
                close $server_stdin;
            }
            next;
        }
        if (fileno($handle) == fileno(STDIN)) {
            write_all($client_trace, $chunk);
            write_all($server_stdin, $chunk);
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
        }
    }
}

waitpid($server_pid, 0);
my $status = $?;
exit 1 if $status == -1;
exit 128 + ($status & 127) if $status & 127;
exit $status >> 8;
"#;
