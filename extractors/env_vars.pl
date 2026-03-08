#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;

sub emit {
  my ($path, $line_no, $raw, $name) = @_;
  my $ev = {
    id => "ev_env_" . $id++,
    record_type => "evidence",
    extractor => "env_vars",
    path => $path,
    line => $line_no,
    raw => $raw,
    captures => { name => $name },
  };
  print encode_json($ev) . "\n";
}

my @files = `find "$root" -type f \\( -name '*.go' -o -name '*.rs' -o -name '*.py' -o -name '*.js' -o -name '*.ts' -o -name '*.rb' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  next if $file =~ /\.git\//;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /(?:getenv|Getenv|process\.env|ENV\{?)\s*\(?\s*["']([A-Z0-9_]{3,})["']/ || $line =~ /ENV\[\s*["']([A-Z0-9_]{3,})["']\s*\]/) {
      my $name = $1;
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      emit($p, $line_no, $line, $name);
    }
  }
  close $fh;
}
