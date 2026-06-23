class ModparksCli < Formula
  desc "CLI for ModParks"
  homepage "https://github.com/Pitan76/ModParks-CLI"
  version "0.0.9"
  
  if OS.mac?
    url "https://github.com/Pitan76/ModParks-CLI/releases/download/v0.0.9/modparks-cli-v0.0.9-macos-x86_64.tar.gz"
    sha256 "d9465dc9e2840f5ad1a67cdb55e5695eb892bc40a2fc8d657dbdb819ba3ac3c8"
  elsif OS.linux?
    url "https://github.com/Pitan76/ModParks-CLI/releases/download/v0.0.9/modparks-cli-v0.0.9-ubuntu-x86_64.tar.gz"
    sha256 "a8f4450d5e2cf7b20917007a4414b0cc703f2b70d9a33cfd231cd1d29b814967"
  end

  def install
    bin.install "modparks-cli"
    bin.install_symlink "modparks-cli" => "modparks-tui"
  end

  test do
    system "#{bin}/modparks-cli", "--version"
  end
end