class Aiproject < Formula
  desc "Project tracking and context management for AI-assisted development"
  homepage "https://github.com/victorysightsound/aiproject"
  version "1.7.17"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-apple-darwin.tar.gz"
      sha256 "5b462ba26c386ab89e390e72d029de04e1ab7d63737cc46eeaad0b12e4c8cd4d"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-apple-darwin.tar.gz"
      sha256 "e2365b57ec955803465e43515bdff98764e82f763646142776085331eda3628c"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "31715126a084b1fb51683a3e9c6f15aa9e794f4085f45a5946939e0226bf6036"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "6ea2d53c0f14f3a48fb15266afdb689b206eafc2a71b126ff357bd9c2f00dad3"
    end
  end

  def install
    bin.install "proj"
  end

  test do
    system "#{bin}/proj", "--version"
  end
end
