class Aiproject < Formula
  desc "Project tracking and context management for AI-assisted development"
  homepage "https://github.com/victorysightsound/aiproject"
  version "1.5.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-apple-darwin.tar.gz"
      sha256 "340f73eae87e5dbb19a1f2f251f198a9cea14e6cd0e2adc7f2361227c587a6de"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-apple-darwin.tar.gz"
      sha256 "a14a550d66b474a134a2101582617aea100541aa8bf60a857f02e0945f143e5f"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "03e997dbe093226fccef9bab6077e8258ac12d7fee03d74e96341510128ef2dc"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "2c031ef76d3b1d30053e1ac577c142640ffa71795dbe382059524b81fd381f47"
    end
  end

  def install
    bin.install "proj"
  end

  test do
    system "#{bin}/proj", "--version"
  end
end
