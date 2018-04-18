# fixsrt
Fix a srt file of common mistakes and spelling errors.

Regarding spelling errors, only french is supported.

Usage:
```
  fixsrt [--nobak] [--out OUTSRTFILE] [--lang LANG] SRTFILE1 SRTFILE2 ...
```

By default, fixsrt will do a backup of your srt file (named by using a ~ suffix).
The --nobak options will prevent fixsrt of doing a backup.

By default, the srt file given as a parameter will be updated. To write to another
srt file, use the --out option.

The default language is french. To select english, use `--lang en`

## How to build on Linux

Install the Cargo build utility that comes with the Rust compiler:
```
sudo apt install cargo
```
Clone the repository:
```
git clone https://github.com/hadrien-psydk/fixsrt
```
Enter the created directory and build with cargo:
```
cd fixsrt
cargo build
```

You can test the executable by running it with Cargo. Examples:
```
cargo run -- --help
cargo run -- subtitles.srt
```
To keep the compiled binary somewhere, prefer building it in release mode:
```
cargo build --release
```
The release executable path will be: target/release/fixsrt. These commands copy the executable
to your home/bin directory, which may conveniently be refered by your $PATH environment variable:
```
mkdir -p ~/bin
cp target/release/fixsrt ~/bin
```







