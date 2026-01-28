class Aiproject < Formula
  desc "Project tracking and context management for AI-assisted development"
  homepage "https://github.com/victorysightsound/aiproject"
  version "1.7.13"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-apple-darwin.tar.gz"
      sha256 "49e5bf6d089c2b04699ae950da750bf04ab7dbdf2dedc9c2ccc8f8299c2adbea"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-apple-darwin.tar.gz"
      sha256 "523393fd7c9939dbeb090cb1b6aa30af4efef091415198e47cb8225941c157ea"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d4d9142aff81689f3b44a253fc4f754622d0e9dda6d6f7ebc6f7de285c5fd3e8"
    end
    on_intel do
      url "https://github.com/victorysightsound/aiproject/releases/download/v#{version}/proj-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "8f52488cef81ec1cf154a64854e047a8506b6ea691f7cc2ed6612db154fa67de"
    end
  end

  def install
    bin.install "proj"
  end

  test do
    system "#{bin}/proj", "--version"
  end
end
