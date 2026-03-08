#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;
my @files = `find "$root" -type f \\( -name '*.go' -o -name '*.rs' -o -name '*.ts' -o -name '*.js' -o -name '*.py' -o -name '*.yaml' -o -name '*.yml' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    while ($line =~ m{(https?://[A-Za-z0-9\./_\-\?=&%#:+]+)}g) {
      my $url = $1;
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      print encode_json({
        id => "ev_url_" . $id++,
        record_type => "evidence",
        extractor => "external_urls",
        path => $p,
        line => $line_no,
        raw => $line,
        captures => { url => $url }
      }) . "\n";
    }
  }
  close $fh;
}
