class Gitviz < Formula
  desc "Terminal Git repository visualizer"
  homepage "https://github.com/emilzmmn04/gitviz"
  version "__VERSION__"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "__MACOS_ARM64_URL__"
      sha256 "__MACOS_ARM64_SHA__"
    else
      url "__MACOS_X64_URL__"
      sha256 "__MACOS_X64_SHA__"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "__LINUX_ARM64_URL__"
      sha256 "__LINUX_ARM64_SHA__"
    else
      url "__LINUX_X64_URL__"
      sha256 "__LINUX_X64_SHA__"
    end
  end

  def install
    bin.install "gitviz"
  end

  test do
    assert_match "gitviz", shell_output("#{bin}/gitviz --help")
  end
end
