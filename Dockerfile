# Start with NVIDIA CUDA base image
FROM nvidia/cuda:12.6.0-devel-ubuntu22.04

# Set the Python version
ARG PYTHON_VERSION=3.10.12

# Set non-interactive mode for tzdata configuration
ENV DEBIAN_FRONTEND=noninteractive

# Update and install base dependencies
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    wget \
    ca-certificates \
    build-essential \
    pkg-config \
    cmake \
    libssl-dev \
    libgmp-dev \
    libffi-dev \
    libsodium-dev \
    nasm \
    git \
    awscli \
    gcc \
    nodejs \
    npm \
    curl \
    m4 \
    software-properties-common \
    zip \
    unzip \
    libbz2-dev \
    liblzma-dev \
    libncursesw5-dev \
    libreadline-dev \
    libsqlite3-dev \
    tk-dev \
    xz-utils \
    zlib1g-dev \
    libomp-dev \
    tzdata && \
    rm -rf /var/lib/apt/lists/*

# Install Bazelisk
RUN wget https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64 && \
    chmod +x bazelisk-linux-amd64 && \
    mv bazelisk-linux-amd64 /usr/local/bin/bazel

# Set HOME environment variable
ENV HOME /root

# Install pyenv and Python
RUN curl https://pyenv.run | bash && \
    export PYENV_ROOT="$HOME/.pyenv" && \
    export PATH="$PYENV_ROOT/bin:$PATH" && \
    CONFIGURE_OPTS=--enable-shared pyenv install ${PYTHON_VERSION} && \
    pyenv global ${PYTHON_VERSION} && \
    pyenv rehash

# Update PATH environment variable
ENV PYENV_ROOT="$HOME/.pyenv"
ENV PATH "$PYENV_ROOT/shims:$PYENV_ROOT/bin:$PATH"

# Install pip and numpy
RUN apt-get update && apt-get install -y --no-install-recommends python3-pip && \
    pip3 install numpy

# Clone tachyon repository
RUN git clone https://github.com/kroma-network/tachyon.git /root/tachyon
WORKDIR /root/tachyon

# Add build:cuda --action_env=TACHYON_CUDA_COMPUTE_CAPABILITIES="compute_52" to .bazelc.user
RUN echo "build:cuda --action_env=TACHYON_CUDA_COMPUTE_CAPABILITIES=\"compute_52\"" >> vendors/circom/.bazelrc

# Build circom prover with CUDA acceleration
RUN cd vendors/circom && \
    TMP=/root CARGO_BAZEL_REPIN=true bazel build --@kroma_network_tachyon//:has_openmp --config maxopt --config cuda //:prover_main

# Create a symlink to the circom prover in /usr/local/bin
RUN ln -s $HOME/tachyon/vendors/circom/bazel-bin/prover_main /usr/local/bin/prover

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH "/root/.cargo/bin:${PATH}"

WORKDIR /prover

# Copy Rust source code and build the project
COPY Cargo.* ./
COPY src ./src
RUN cargo build

# Reset DEBIAN_FRONTEND to default value
ENV DEBIAN_FRONTEND=

# Set the default command to open a bash shell
CMD ["cargo", "run"]
