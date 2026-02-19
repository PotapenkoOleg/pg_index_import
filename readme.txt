# AlmaLinux packages
sudo curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo dnf install openssl openssl-devel
sudo dnf groupinstall "Development Tools"