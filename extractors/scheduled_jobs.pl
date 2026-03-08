#!/usr/bin/env perl
use strict;
use warnings;
use JSON::PP qw(encode_json);

my $root = $ARGV[0] // '.';
my $id = 0;
my @files = `find "$root" -type f \\( -name '*.yaml' -o -name '*.yml' -o -name '*.go' -o -name '*.rs' -o -name '*.js' -o -name '*.ts' -o -name '*.rb' -o -name 'crontab*' \\) 2>/dev/null`;
for my $file (@files) {
  chomp $file;
  open my $fh, '<', $file or next;
  my $line_no = 0;
  while (my $line = <$fh>) {
    $line_no++;
    if ($line =~ /(cron|schedule|\@daily|\@hourly|\*\s+\*\s+\*\s+\*\s+\*)/i) {
      (my $p = $file) =~ s/^\Q$root\E\/?//;
      print encode_json({
        id => "ev_sched_" . $id++,
        record_type => "evidence",
        extractor => "scheduled_jobs",
        path => $p,
        line => $line_no,
        raw => $line,
        captures => { marker => "schedule" }
      }) . "\n";
    }
  }
  close $fh;
}
