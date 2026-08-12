class Metadata < Formula
  desc "Media file metadata for human consumption"
  homepage "https://github.com/zmwangx/metadata"
  url "https://github.com/zmwangx/metadata/archive/v0.1.13.tar.gz"
  sha256 "19bb33ec0a69d0ab757a2334bfafadce3ea017995f3542e94e3defc7148b1849"

  depends_on "pkg-config" => :build
  depends_on "rust" => :build
  depends_on "ffmpeg-full"

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
