# Frame count:
# - 020FFA_09936E53EA4BC_001.ppm: 44
# - 022FE6_0987E2DB2322A_003.ppm: 244

cargo build --release

# Feel free to uncomment ppm-parser's benchmark, however, it takes a VERY LONG
# time to complete, especially with three warmup runs.
#
# PPM-Parser Benchmark + (3 * PPM-Parser Warmup Runs) ~ (~50.4 + (3 * ~50.4)
hyperfine --warmup 3 `
  ".\\target\\release\\para .\\benches\\ppm\\022FE6_0987E2DB2322A_003.ppm gif bench.gif" # `
  # "py .\\benches\\ppm-parser\\ppmImage.py .\\benches\\ppm\\022FE6_0987E2DB2322A_003.ppm gif bench.gif"
hyperfine --warmup 3 `
  ".\\target\\release\\para .\\benches\\ppm\\022FE6_0987E2DB2322A_003.ppm 0 bench.png" `
  "py .\\benches\\ppm-parser\\ppmImage.py .\\benches\\ppm\\022FE6_0987E2DB2322A_003.ppm 0 bench.png"

Remove-Item -ErrorAction Ignore bench.gif
Remove-Item -ErrorAction Ignore bench.png
