#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;
my @files = `find "$root" -type f \\( -name '*.proto' -o -name '*.go' -o -name '*.rs' -o -name '*.js' -o -name '*.ts' -o -name '*.rb' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /\bservice\s+([A-Za-z0-9_]+)/ || $line =~ /grpc[^"']*["']([A-Za-z0-9_.]+)/i) {
      my $name = $1;
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      print encode_json({
        id => "ev_rpc_" . $id++,
        record_type => "evidence",
        extractor => "rpc_services",
        path => $p,
        line => $line_no,
        raw => $line,
        captures => { name => $name }
      }) . "\n";
    }
  }
  close $fh;
}
