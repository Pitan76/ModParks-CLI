class ModparksCli < Formula
  desc "CLI for ModParks"
  homepage "https://github.com/Pitan76/ModParks"
  version "0.1.0"
  
  if OS.mac?
    url "https://github.com/Pitan76/ModParks/releases/download/v0.1.0/modparks-cli-v0.1.0-macos-x86_64.tar.gz"
    sha256 "REPLACE_WITH_SHA256"
  elsif OS.linux?
    url "https://github.com/Pitan76/ModParks/releases/download/v0.1.0/modparks-cli-v0.1.0-ubuntu-x86_64.tar.gz"
    sha256 "REPLACE_WITH_SHA256"
  end

  def install
    bin.install "modparks-cli"
  end

  test do
    system "#{bin}/modparks-cli", "--version"
  end
end