class Aiproject < Formula
  desc "Project tracking and context management for AI-assisted development"
  homepage "https://github.com/victorysightsound/aiproject"
  version "1.6.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-apple-darwin.tar.gz"
      sha256 "ccae0326a6ab9460c5ad18490f7aee86f2da78d75369d893acaa63afbc49f34d"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-apple-darwin.tar.gz"
      sha256 "d0984411304c97edcdfe402a7423a5acb1a921484ae78291a41a5be3b7b7ab61"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "c5a1f53f5bbfbe8cdf71cf6e7ef4c8cf25f5f3a35c9b815662eb8dcb4acb3903"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e2a6320cb945a900d4bbb3b38501f9607298bdb5433c53a1f9f1d91073a6d9b8"
    end
  end

  def install
    bin.install "proj"
  end

  test do
    system "#{bin}/proj", "--version"
  end
end
