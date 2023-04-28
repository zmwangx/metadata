class Metadata < Formula
  desc "Media file metadata for human consumption"
  homepage "https://github.com/zmwangx/metadata"
  url "https://github.com/zmwangx/metadata/archive/v0.1.8.tar.gz"
  sha256 "135d02ac45093e6af9405aeb95544c03283da8cb2c0a88fec5de4b85e0aa5b40"

  depends_on "pkg-config" => :build
  depends_on "rust" => :build
  depends_on "ffmpeg"

  def install
    system "make", "release"
    bin.install "dist/v#{version}/metadata"
    man1.install "dist/v#{version}/metadata.1"
  end

  test do
    cp test_fixtures("test.mp3"), "test.mp3"
    assert_match(/Filename:\s+test.mp3.*Container format:\s+MP3/m,
                 shell_output("#{bin}/metadata test.mp3"))
  end
end
