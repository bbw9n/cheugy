#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;

sub emit {
  my ($path, $line_no, $raw, $topic) = @_;
  my $ev = {
    id => "ev_queue_" . $id++,
    record_type => "evidence",
    extractor => "queues",
    path => $path,
    line => $line_no,
    raw => $raw,
    captures => { topic => $topic },
  };
  print encode_json($ev) . "\n";
}

my @files = `find "$root" -type f \\( -name '*.go' -o -name '*.rs' -o -name '*.py' -o -name '*.js' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /(kafka|topic|queue|publish|consume)[^"']*["']([a-zA-Z0-9_.-]+)["']/i) {
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      emit($p, $line_no, $line, $2);
    }
  }
  close $fh;
}
