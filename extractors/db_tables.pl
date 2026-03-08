#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;

sub emit {
  my ($path, $line_no, $raw, $table) = @_;
  my $ev = {
    id => "ev_table_" . $id++,
    record_type => "evidence",
    extractor => "db_tables",
    path => $path,
    line => $line_no,
    raw => $raw,
    captures => { table => $table },
  };
  print encode_json($ev) . "\n";
}

my @files = `find "$root" -type f \\( -name '*.sql' -o -name '*.go' -o -name '*.rs' -o -name '*.py' -o -name '*.js' -o -name '*.ts' -o -name '*.rb' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  next if $file =~ /\.git\//;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /\b(select|insert|update|delete|create\s+table)\b/i) {
      if ($line =~ /\b(?:from|join|into|update|table)\s+([a-zA-Z_][a-zA-Z0-9_]*)/i) {
        my $table = $1;
        (my $p = $file) =~ s/^\Q$root\E\/?//;
        emit($p, $line_no, $line, $table);
      }
    }
  }
  close $fh;
}
