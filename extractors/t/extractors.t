use strict;
use warnings;
use Test::More;
use JSON::PP qw(decode_json);
use File::Temp qw(tempdir);
use File::Path qw(make_path);

my $repo = tempdir(CLEANUP => 1);

write_file(
  'src/main.rs',
  <<'EOF'
fn main() {
    println!("boot");
}
EOF
);

write_file(
  'src/app.ts',
  <<'EOF'
if (require.main === module) {
  console.log("boot");
}

router.POST("/payments/:id", handlePayment);
const apiBase = "https://api.example.com/v1";
const topicName = kafka.publish("billing.events");
const flag = feature_flag("checkout_redesign");
const counter = prometheus.counter("billing_requests_total");
const query = "select * from invoices";
const grpcClient = grpc.connect("BillingService");
const cron = "@hourly";
EOF
);

write_file(
  'src/config.py',
  <<'EOF'
import os

token = os.getenv("PAYMENTS_API_KEY")

if __name__ == "__main__":
    print(token)
EOF
);

write_file(
  'src/app.rb',
  <<'EOF'
if __FILE__ == $PROGRAM_NAME
  puts "boot"
end

post "/ruby-payments/:id" do
  ENV["RUBY_API_KEY"]
  feature_flag("ruby_rollout")
  prometheus.counter("ruby_requests_total")
  kafka.publish("ruby.events")
  grpc.connect("RubyBillingService")
  Faraday.get("https://ruby.example.com/v1")
  sql = "select * from ruby_invoices"
  cron = "@daily"
end
EOF
);

write_file(
  'proto/billing.proto',
  <<'EOF'
service BillingService {
  rpc Charge (ChargeRequest) returns (ChargeReply);
}
EOF
);

write_file(
  'deploy/cron.yaml',
  <<'EOF'
schedule: "0 * * * *"
EOF
);

my %cases = (
  entrypoints => [
    [ 'src/main.rs', kind => 'entrypoint' ],
    [ 'src/app.ts', kind => 'entrypoint' ],
    [ 'src/app.rb', kind => 'entrypoint' ],
  ],
  http_routes => [
    [ 'src/app.ts', route => '/payments/:id' ],
    [ 'src/app.rb', route => '/ruby-payments/:id' ],
  ],
  env_vars => [
    [ 'src/config.py', name => 'PAYMENTS_API_KEY' ],
    [ 'src/app.rb', name => 'RUBY_API_KEY' ],
  ],
  db_tables => [
    [ 'src/app.ts', table => 'invoices' ],
    [ 'src/app.rb', table => 'ruby_invoices' ],
  ],
  queues => [
    [ 'src/app.ts', topic => 'billing.events' ],
    [ 'src/app.rb', topic => 'ruby.events' ],
  ],
  rpc_services => [
    [ 'proto/billing.proto', name => 'BillingService' ],
    [ 'src/app.ts', name => 'BillingService' ],
    [ 'src/app.rb', name => 'RubyBillingService' ],
  ],
  scheduled_jobs => [
    [ 'deploy/cron.yaml', marker => 'schedule' ],
    [ 'src/app.ts', marker => 'schedule' ],
    [ 'src/app.rb', marker => 'schedule' ],
  ],
  feature_flags => [
    [ 'src/app.ts', name => 'checkout_redesign' ],
    [ 'src/app.rb', name => 'ruby_rollout' ],
  ],
  metrics => [
    [ 'src/app.ts', name => 'billing_requests_total' ],
    [ 'src/app.rb', name => 'ruby_requests_total' ],
  ],
  external_urls => [
    [ 'src/app.ts', url => 'https://api.example.com/v1' ],
    [ 'src/app.rb', url => 'https://ruby.example.com/v1' ],
  ],
);

for my $extractor (sort keys %cases) {
  my $records = run_extractor($extractor);
  ok(@$records > 0, "$extractor emits evidence");

  for my $case (@{ $cases{$extractor} }) {
    my ($path, $capture_key, $capture_value) = @$case;
    ok(
      has_record($records, $extractor, $path, $capture_key, $capture_value),
      "$extractor finds $path with $capture_key=$capture_value",
    );
  }
}

done_testing();

sub run_extractor {
  my ($extractor) = @_;
  my $script = "extractors/$extractor.pl";
  my $output = qx{perl "$script" "$repo"};
  my $exit = $? >> 8;
  is($exit, 0, "$extractor exits cleanly") or diag($output);

  return [
    map { decode_json($_) }
    grep { /\S/ }
    split /\n/, $output
  ];
}

sub has_record {
  my ($records, $extractor, $path, $capture_key, $capture_value) = @_;

  for my $record (@$records) {
    next unless $record->{extractor} eq $extractor;
    next unless $record->{path} eq $path;
    next unless exists $record->{captures}{$capture_key};
    next unless defined $record->{captures}{$capture_key};
    return 1 if $record->{captures}{$capture_key} eq $capture_value;
  }

  return 0;
}

sub write_file {
  my ($relative, $contents) = @_;
  my $full = "$repo/$relative";
  my ($dir) = $full =~ m{^(.*)/[^/]+$};
  make_path($dir) if defined $dir;

  open my $fh, '>', $full or die "open $full: $!";
  print {$fh} $contents;
  close $fh or die "close $full: $!";
}
