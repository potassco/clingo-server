# 1. Stage: Building clingo-server
FROM continuumio/miniconda3

# Updating & installing build dependencies
RUN apt-get update -y && apt-get install -y \
    build-essential \
    git \
    curl 

RUN conda install -c potassco/label/dev clingo
RUN conda install -c potassco/label/dev clingo-dl
RUN conda install -c potassco/label/dev clingcon

ENV CLINGO_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGO_DL_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGCON_LIBRARY_PATH="/opt/conda/lib"
ENV LD_LIBRARY_PATH="/opt/conda/lib"

# Install Rust
RUN curl https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN git clone https://github.com/potassco/clingo-server.git && \
    cd clingo-server && \
    cargo build --release && \
    mkdir -p /install/bin && \
    cp target/release/cl-server /install/bin/

# 2. Stage: Setup clingo-server run image
FROM continuumio/miniconda3

# Install libraries for clingo, clingo-dl and clingcon
RUN conda install -c potassco/label/dev clingo
RUN conda install -c potassco/label/dev clingo-dl
RUN conda install -c potassco/label/dev clingcon

ENV CLINGO_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGO_DL_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGCON_LIBRARY_PATH="/opt/conda/lib"
ENV LD_LIBRARY_PATH="/opt/conda/lib"

# Copy clingo-server executable from the previous stage across
COPY --from=builder /install /install
