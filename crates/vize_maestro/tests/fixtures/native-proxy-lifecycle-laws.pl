use strict;
use warnings;
use Digest::SHA;
use File::Copy;
use File::Temp qw(tempdir);
use IPC::Open3;
use JSON::PP;
use Symbol qw(gensym);
use Test::More;

my ($fixtures, $source) = @ARGV;
require "$fixtures/native-proxy-lifecycle.pl";
my $json = JSON::PP->new->utf8->canonical;
sub read_bytes {
    open my $file, '<:raw', $_[0] or die "read whole fixture: $!";
    local $/;
    return <$file>;
}
sub frame {
    my $body = $json->encode($_[0]);
    return 'Content-Length: ' . length($body) . "\r\n\r\n" . $body;
}
my $expected = $json->decode(read_bytes("$fixtures/native-proxy-initialize.json"));
my $init = { jsonrpc => '2.0', id => 11, method => 'initialize', params => {} };
my $shutdown = { jsonrpc => '2.0', id => 12, method => 'shutdown' };
my $exit = { jsonrpc => '2.0', method => 'exit' };
my $init_response = { jsonrpc => '2.0', id => 11, result => $expected };
my $ack = { jsonrpc => '2.0', id => 12, result => undef };

# These framing laws deliberately use an explicitly synthetic process seam.
# The foreign executable control below uses the complete actual proxy/process.
sub parser {
    return bless { candidate => 1, expected => $expected,
        pending => { client => '', server => '' }, errors => [],
        frames => { client => [], server => [] }, translated => 0 }, 'NativeProxyLifecycle';
}
sub send_frames {
    my ($p, $side, $bytes) = @_;
    my @frames = $p->frames($side, $bytes);
    for my $f (@frames) {
        $side eq 'client' ? $p->client(@$f, 71) : $p->server($f->[1], 71);
    }
    return @frames;
}
sub initialized {
    my $p = parser();
    send_frames($p, 'client', frame($init));
    send_frames($p, 'server', frame($init_response));
    send_frames($p, 'client', frame($shutdown));
    send_frames($p, 'server', frame($ack));
    return $p;
}
{
    no warnings 'redefine';
    local *NativeProxyLifecycle::process_identity = sub { return { pid => 71, birth => 91 } };
    my $whole = frame($init) . frame($shutdown);
    for my $split (1 .. length($whole) - 1) {
        my $p = parser();
        my @got = ($p->frames('client', substr($whole, 0, $split)),
            $p->frames('client', substr($whole, $split)));
        is(join('', map { $_->[0] } @got), $whole, "whole coalesced frames survive split $split");
        is_deeply([map { $_->[1] } @got], [$init, $shutdown], "both complete messages at split $split");
    }
    my $p = initialized();
    ok($p->{initialized} && $p->{acknowledged}, 'full initialize contract and matching null ACK admitted');
    my $whole_exit = frame($exit);
    my @fragment = $p->frames('client', substr($whole_exit, 0, -1));
    is(scalar @fragment, 0, 'partial Exit is never translated');
    my @complete = $p->frames('client', substr($whole_exit, -1));
    is($p->client(@{$complete[0]}, 71), 1, 'only complete Exit translates after authenticated ACK');
    is($p->{wholeExitFrameHex}, unpack('H*', $whole_exit), 'complete Exit frame retained byte-exact');
    for my $case (
        ['request Exit', { %$exit, id => 13 }],
        ['null Exit params', { %$exit, params => undef }],
        ['Exit params', { %$exit, params => {} }],
        ['extra Exit member', { %$exit, extra => 1 }],
    ) {
        my $p = initialized();
        my @f = $p->frames('client', frame($case->[1]));
        is($p->client(@{$f[0]}, 71), 0, "$case->[0] preserves ordinary forwarding");
    }
    for my $case (
        ['wrong whole contract', { %$init_response, result => { %$expected, extra => 1 } }, $ack],
        ['wrong ACK identity', $init_response, { %$ack, id => '12' }],
        ['non-null ACK', $init_response, { %$ack, result => 0 }],
        ['error ACK', $init_response, { %$ack, error => { code => -1, message => 'whole refusal' } }],
    ) {
        my $p = parser();
        send_frames($p, 'client', frame($init));
        send_frames($p, 'server', frame($case->[1]));
        send_frames($p, 'client', frame($shutdown));
        send_frames($p, 'server', frame($case->[2]));
        my @f = $p->frames('client', frame($exit));
        is($p->client(@{$f[0]}, 71), 0, "$case->[0] cannot translate Exit");
    }
    my $duplicate = initialized();
    send_frames($duplicate, 'client', frame($shutdown));
    send_frames($duplicate, 'server', frame($ack));
    my @f = $duplicate->frames('client', frame($exit));
    is($duplicate->client(@{$f[0]}, 71), 0, 'duplicate shutdown cannot acquire capability');
    like($duplicate->{errors}[0]{reason}, qr/duplicate complete shutdown/, 'duplicate whole refusal retained');
    my $duplicate_init = initialized();
    send_frames($duplicate_init, 'client', frame($init));
    send_frames($duplicate_init, 'server', frame($init_response));
    my @duplicate_init = $duplicate_init->frames('client', frame($exit));
    is($duplicate_init->client(@{$duplicate_init[0]}, 71), 0, 'duplicate initialize cannot acquire capability');
    for my $bad (
        "Content-Length: 1\r\nContent-Length: 1\r\n\r\n{}",
        "Content-Length: 1\r\n\r\nx",
        "Content-Length: 536870913\r\n\r\n",
    ) {
        my $p = initialized();
        my @f = $p->frames('client', $bad);
        ok(@{$p->{errors}}, 'malformed whole input retains explicit refusal');
        is($p->{errors}[0]{wholeBytesHex}, unpack('H*', $bad), 'entire malformed packet retained');
        ok(!$p->{translated}, 'malformed input does not acquire clean termination');
    }
    my $partial = initialized();
    $partial->frames('client', substr($whole_exit, 0, -1));
    $partial->eof('client');
    is($partial->{errors}[0]{wholeBytesHex}, unpack('H*', substr($whole_exit, 0, -1)),
        'EOF inside a frame retains complete observed bytes');
    my $tail = initialized();
    send_frames($tail, 'client', $whole_exit);
    send_frames($tail, 'client', frame($shutdown));
    like($tail->{errors}[0]{reason}, qr/after translated Exit/, 'post-Exit frame refuses clean termination');
    my $partial_tail = initialized();
    send_frames($partial_tail, 'client', $whole_exit . substr(frame($shutdown), 0, -1));
    $partial_tail->retire_client_reader();
    is($partial_tail->{errors}[0]{wholeBytesHex}, unpack('H*', substr(frame($shutdown), 0, -1)),
        'retired client reader retains the whole observed partial post-Exit packet');
    my $changed_birth = initialized();
    {
        local *NativeProxyLifecycle::process_identity = sub { return { pid => 71, birth => 92 } };
        my @changed = $changed_birth->frames('client', $whole_exit);
        is($changed_birth->client(@{$changed[0]}, 71), 0, 'same numeric PID with a different birth cannot translate Exit');
    }
    my $dead = initialized();
    local *NativeProxyLifecycle::process_identity = sub { return undef };
    my @dead = $dead->frames('client', $whole_exit);
    is($dead->client(@{$dead[0]}, 71), 0, 'missing actual owned image cannot translate Exit');
}

my $root = tempdir(CLEANUP => 1);
for my $name ('native-proxy-lifecycle.pl', 'native-proxy-initialize.json') {
    copy("$fixtures/$name", "$root/$name") or die "copy capability fixture: $!";
}
my $rust = read_bytes($source);
$rust =~ /r#"(.*)"#;/s or die 'missing entire source proxy';
open my $proxy, '>:raw', "$root/proxy.pl" or die "write proxy: $!";
print {$proxy} $1;
close $proxy;
my $foreign = <<'FOREIGN';
#!/usr/bin/perl
use strict;
use warnings;
use JSON::PP;
my ($capture, $contract) = @ARGV;
open my $raw, '>:raw', $capture or die $!;
open my $expected, '<:raw', $contract or die $!;
local $/;
my $whole_contract = <$expected>;
local $/ = "\n";
while (my $header = <STDIN>) {
    $header =~ /Content-Length: (\d+)/ or die 'foreign frame header';
    my $length = $1;
    my $blank = <STDIN>;
    my $body = '';
    while (length($body) < $length) {
        my $read = read STDIN, my $part, $length - length($body);
        die 'foreign partial body' unless $read;
        $body .= $part;
    }
    print {$raw} $header, $blank, $body;
    my $m = decode_json($body);
    if (exists $m->{id}) {
        my $result = $m->{method} eq 'initialize' ? decode_json($whole_contract) : undef;
        my $reply = encode_json({ jsonrpc => '2.0', id => $m->{id}, result => $result });
        print STDOUT 'Content-Length: ', length($reply), "\r\n\r\n", $reply;
    }
}
close $raw;
print STDERR "whole foreign stderr\nwith second line\n";
exit 17;
FOREIGN
open my $fake, '>:raw', "$root/foreign.pl" or die "write foreign: $!";
print {$fake} $foreign;
close $fake;
chmod 0755, "$root/foreign.pl" or die "chmod foreign: $!";
my $stderr = gensym;
my ($stdin, $stdout);
my $pid = open3($stdin, $stdout, $stderr, 'perl', "$root/proxy.pl",
    "$root/client.raw", "$root/server.raw", "$root/native.stderr", "$root/no-gate", 0,
    "$root/foreign.pl", "$root/native.raw", "$root/native-proxy-initialize.json");
my $all = frame($init) . frame($shutdown) . frame($exit);
print {$stdin} $all;
close $stdin;
local $/;
my $whole_stdout = <$stdout>;
my $whole_stderr = <$stderr>;
waitpid($pid, 0);
is($? >> 8, 17, 'actual foreign process exit17 is propagated unchanged');
is($whole_stderr, '', 'no synthetic transport warning conceals actual native refusal');
is(read_bytes("$root/native.stderr"), "whole foreign stderr\nwith second line\n", 'full native stderr survives real Open3 ownership');
is(read_bytes("$root/native.raw"), $all, 'unknown executable receives complete original initialize/shutdown/Exit');
is(read_bytes("$root/client.raw"), $all, 'whole client trace remains exact');
is(read_bytes("$root/server.raw"), $whole_stdout, 'whole native responses remain byte-exact');
my $receipt = $json->decode(read_bytes("$root/native.stderr.lifecycle.json"));
is($receipt->{candidate}, 0, 'matching fake native contract cannot authenticate foreign executable bytes');
is($receipt->{translated}, 0, 'unknown backend retains standard Shutdown/Exit');
is($receipt->{nativeWaitStatus}, 17 << 8, 'actual complete native wait status is retained');
is($receipt->{reapedNativePid}, $receipt->{nativePid}, 'waitpid actually reaps the original foreign native child');
note('Whole foreign process refusal packet: ' . $json->encode({ lifecycle => $receipt,
    wholeClientHex => unpack('H*', read_bytes("$root/client.raw")),
    wholeNativeInputHex => unpack('H*', read_bytes("$root/native.raw")),
    wholeNativeStdoutHex => unpack('H*', $whole_stdout),
    wholeNativeStderrHex => unpack('H*', read_bytes("$root/native.stderr")),
    foreignBodySha256 => Digest::SHA::sha256_hex($foreign),
    wholeProxySha256 => Digest::SHA::sha256_hex(read_bytes("$root/proxy.pl")) }));
done_testing();
