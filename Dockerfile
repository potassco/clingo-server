FROM continuumio/miniconda3

# Updating & installing necessary packages
RUN apt-get update -y && apt-get install -y \
    git \
    curl 

RUN conda install -c potassco/label/dev clingo
RUN conda install -c potassco/label/dev clingo-dl
RUN conda install -c potassco/label/dev clingcon

RUN conda env list 

ENV CLINGO_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGO_DL_LIBRARY_PATH="/opt/conda/lib"
ENV CLINGCON_LIBRARY_PATH="/opt/conda/lib"
ENV LD_LIBRARY_PATH="/opt/conda/lib"


RUN curl https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN git clone https://github.com/potassco/clingo-server.git && \
    cd clingo-server && \
    cargo build --release && \
    mkdir -p /install/bin && \
    cp target/release/cl-server /install/bin/
