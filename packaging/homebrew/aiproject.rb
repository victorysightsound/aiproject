class Aiproject < Formula
  desc "Project tracking and context management for AI-assisted development"
  homepage "https://github.com/victorysightsound/aiproject"
  version "1.8.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-apple-darwin.tar.gz"
      sha256 "e289c06c45edbae50acef5789cd9e8336c117ac10cf6f9b0590d4170acf5f93c"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-apple-darwin.tar.gz"
      sha256 "44080c9ae6fbfc1b9fe9c35db197239001c05d243f78f256265f77ef811b6c52"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "8ca6c796f35de64ba012e67a0336a191df22adfa3f58c5b6587ec25d281e1ac7"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "a4b6f5c390f4cc5c493a813b3e848a3c75c73a1fce62526864d5a971189c660e"
    end
  end

  def install
    bin.install "proj"
  end

  test do
    system "#{bin}/proj", "--version"
  end
end
