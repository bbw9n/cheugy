#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;

sub emit {
  my ($path, $line_no, $raw, $method, $route) = @_;
  my $ev = {
    id => "ev_route_" . $id++,
    record_type => "evidence",
    extractor => "http_routes",
    path => $path,
    line => $line_no,
    raw => $raw,
    captures => { method => $method, route => $route },
  };
  print encode_json($ev) . "\n";
}

my @files = `find "$root" -type f \\( -name '*.go' -o -name '*.js' -o -name '*.ts' -o -name '*.rs' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  next if $file =~ /\.git\//;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /(GET|POST|PUT|DELETE|PATCH)\s*\(\s*["']([^"']+)["']/i) {
      my ($method, $route) = (uc($1), $2);
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      emit($p, $line_no, $line, $method, $route);
    }
  }
  close $fh;
}
