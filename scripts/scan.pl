#!/usr/bin/env perl
use strict;
use warnings;

my $root = $ARGV[0] // '.';
my @extractors = glob("extractors/*.pl");
for my $ex (@extractors) {
  next if $ex =~ /manifest\.json$/;
  system('perl', $ex, $root) == 0 or die "extractor failed: $ex\n";
}
