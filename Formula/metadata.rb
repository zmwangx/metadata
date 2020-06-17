class Metadata < Formula
  desc "Media file metadata for human consumption"
  homepage "https://github.com/zmwangx/metadata"
  url "https://github.com/zmwangx/metadata/archive/v0.1.4.tar.gz"
  sha256 "b07a78fb85d0efd9cc7ed186407c7bc3a76474dd58d5cde88d8ed58bddbd31fc"

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
