# clingo-server

Install clingo and clingo-dl development version

```sh
conda install -c potassco/label/dev clingo clingo-dl
```

Set paths to clingo and clingo-dl library `libclingo.so` `libclingo-dl.so`

```sh
export CLINGO_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
export CLINGO_DL_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
export LD_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
```

Start the server with

```sh
cargo +nightly run
```

Test the server with

```sh
python client.py -i queens.lp
```

or

```sh
python client.py -i pigeonhole.lp
```

or with dl constraints

```sh
python client.py -i taskassignment.lp
```
