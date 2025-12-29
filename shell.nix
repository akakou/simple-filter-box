{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.gnuplot
    pkgs.python3
  ];
  shellHook = ''
    export NIX_ENFORCE_PURITY=0
    CRITERION_DEBUG=1 cargo bench --bench filter_bench -- --plotting-backend gnuplot
    mkdir -p ./target/criterion/plots/
    cp ./target/criterion/test/bloom_miss/report/lines.gnuplot ./target/criterion/plots/test_bloom_miss.gnuplot
    cp ./target/criterion/test/redb_miss/report/lines.gnuplot ./target/criterion/plots/test_redb_miss.gnuplot
    cp ./target/criterion/insert/bloom/report/lines.gnuplot ./target/criterion/plots/insert_bloom.gnuplot
    cp ./target/criterion/insert/redb/report/lines.gnuplot ./target/criterion/plots/insert_redb.gnuplot

    cd ./target/criterion/plots/

    sed -i '/^set grid ytics$/a\set yrange [0:*]' ./test_bloom_miss.gnuplot
    sed -i '/^set grid ytics$/a\set yrange [0:*]' ./test_redb_miss.gnuplot
    sed -i '/^set grid ytics$/a\set yrange [0:*]' ./insert_bloom.gnuplot
    sed -i '/^set grid ytics$/a\set yrange [0:*]' ./insert_redb.gnuplot

    gnuplot ./test_bloom_miss.gnuplot
    mv output.plot ./test_bloom_miss.svg
    gnuplot ./test_redb_miss.gnuplot 
    mv output.plot ./test_redb_miss.svg

    gnuplot ./insert_bloom.gnuplot
    mv output.plot ./insert_bloom.svg
    gnuplot ./insert_redb.gnuplot
    mv output.plot ./insert_redb.svg
  '';
}
