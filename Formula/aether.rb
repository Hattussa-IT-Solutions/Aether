class Aether < Formula
  desc "Modern programming language with parallelism, pattern matching, and genetic evolution"
  homepage "https://aether-lang.org"
  license "Apache-2.0"

  # Update these URLs and checksums for each release
  on_macos do
    on_arm do
      url "https://github.com/aether-lang/aether/releases/latest/download/aether-macos-aarch64.tar.gz"
      sha256 "UPDATE_SHA256_HERE"
    end
    on_intel do
      url "https://github.com/aether-lang/aether/releases/latest/download/aether-macos-x86_64.tar.gz"
      sha256 "UPDATE_SHA256_HERE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/aether-lang/aether/releases/latest/download/aether-linux-aarch64.tar.gz"
      sha256 "UPDATE_SHA256_HERE"
    end
    on_intel do
      url "https://github.com/aether-lang/aether/releases/latest/download/aether-linux-x86_64.tar.gz"
      sha256 "UPDATE_SHA256_HERE"
    end
  end

  def install
    bin.install "aether"
  end

  test do
    assert_match "aether", shell_output("#{bin}/aether --version")

    (testpath/"hello.ae").write('print("Hello from Aether!")')
    assert_match "Hello from Aether!", shell_output("#{bin}/aether run #{testpath}/hello.ae")
  end
end
