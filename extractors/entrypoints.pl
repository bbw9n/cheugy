#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;

sub emit {
  my ($path, $line_no, $raw, $captures) = @_;
  my $ev = {
    id => "ev_entry_" . $id++,
    record_type => "evidence",
    extractor => "entrypoints",
    path => $path,
    line => $line_no,
    raw => $raw,
    captures => $captures,
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
    if (
      $line =~ /\bfunc\s+main\s*\(/
      || $line =~ /\bfn\s+main\s*\(/
      || $line =~ /if\s+__name__\s*==\s*["']__main__["']/
      || $line =~ /require\.main\s*===\s*module/
      || $line =~ /if\s+__FILE__\s*==\s*\$PROGRAM_NAME/
    ) {
      $file =~ s/^\Q$root\E\/?//;
      emit($file, $line_no, $line, { kind => "entrypoint" });
    }
  }
  close $fh;
}
