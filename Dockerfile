# Use Node.js 20 with a slim Debian base image
FROM node:20-bullseye-slim

# Install curl and required dependencies for Tauri
RUN apt-get update && \
    apt-get install -y curl wget file libgtk-3-dev libssl-dev pkg-config \
    libwebkit2gtk-4.0-dev build-essential libxdo-dev libayatana-appindicator3-dev librsvg2-dev && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    apt-get clean

# Set up the environment for Rust
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a non-root user with a home directory
RUN useradd -m dockeruser

# Set up the working directory and ensure ownership by dockeruser
WORKDIR /app
COPY . .
RUN chown -R dockeruser:dockeruser /app

# Switch to non-root user
USER dockeruser

# Install Tauri CLI locally and dependencies
RUN npm install @tauri-apps/cli && npm install

# Expose ports
EXPOSE 1420
EXPOSE 14550

# Start the Tauri app in development mode
CMD ["npx", "tauri", "dev"]

