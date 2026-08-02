class Fingerprint < Formula
  desc "Winnowing document fingerprinting and Jaccard similarity — detect duplicated logic"
  homepage "https://github.com/grahambrooks/fingerprint-rs"
  version "2026.7.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/fingerprint-rs/archive/refs/tags/v2026.8.1.tar.gz"
      sha256 "df9bd3f4599198e42ea77b8484718a4e8d9c6838becd14a3460be894efa01afc"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/grahambrooks/fingerprint-rs --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/grahambrooks/fingerprint-rs/releases/download/v2026.7.0/fingerprint-v2026.7.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "8b477aae49cfab10651005c59e5460955e998b6c71c1e28297dcfede40547725"
    end
    on_intel do
      url "https://github.com/grahambrooks/fingerprint-rs/releases/download/v2026.7.0/fingerprint-v2026.7.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "8ead9598713c8801c93fea918a9c51d06f1a219a37da643bb05bb678caae3a06"
    end
  end

  def install
    bin.install "fingerprint"
  end

  test do
    assert_path_exists bin/"fingerprint"
  end
end
