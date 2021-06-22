# clingo-server

Install clingo, clingo-dl and clingcon development versions.

```sh
conda install -c potassco/label/dev clingo clingo-dl clingcon
```

Set paths to clingo and other theory plugin libraries `libclingo.so`  `libclingo-dl.so` and `libclingcon.so`

```sh
export CLINGO_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
export CLINGO_DL_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
export CLINGCON_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
export LD_LIBRARY_PATH=~/miniconda3/envs/cl-server/lib
```

Start the server with

```sh
cargo run
```

Test the server with

```sh
python client.py -i queens.lp --assume
```

or

```sh
python client.py -i pigeonhole.lp --pigeons
```

or with dl constraints

```sh
python client.py -i taskassignment.lp --theory-dl
```

or with clingcon constraints

```sh
python client.py -i golomb.lp --theory-con
```


[API documentation](API.md)
