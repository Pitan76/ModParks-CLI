class ModparksCli < Formula
  desc "CLI for ModParks"
  homepage "https://github.com/Pitan76/ModParks-CLI"
  version "0.0.8"
  
  if OS.mac?
    url "https://github.com/Pitan76/ModParks-CLI/releases/download/v0.0.1/modparks-cli-v0.0.1-macos-x86_64.tar.gz"
    sha256 "REPLACE_WITH_SHA256"
  elsif OS.linux?
    url "https://github.com/Pitan76/ModParks-CLI/releases/download/v0.0.1/modparks-cli-v0.0.1-ubuntu-x86_64.tar.gz"
    sha256 "REPLACE_WITH_SHA256"
  end

  def install
    bin.install "modparks-cli"
    bin.install_symlink "modparks-cli" => "modparks-tui"
  end

  test do
    system "#{bin}/modparks-cli", "--version"
  end
end