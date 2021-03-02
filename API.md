# Clingo server API

## Creating a solver

Method: `GET` 

```url
curl http://localhost:8000/create
```
### Responses

Status: 200 OK 
```
Created clingo solver.
```
```json
{
    "type": "InternalError",
    "msg": "Solver::create failed! Solver still running!"
}
```

## Register a (DL) theory

Method: `GET` 

```url
curl http://localhost:8000/register_dl_theory
```
### Responses

Status: 200 OK 

```
Difference logic theory registered.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::register_dl_theory failed! DLTheory already registered."
}
```
```json
{
    "type": "InternalError",
    "msg": "Solver::register_dl_theory failed! Solver has been already started."
}
```

## Add a logic program

Method: `POST` 

```url
curl -i -XPOST http://localhost:8000/add --header 'content-type:text/plain' --data 'p:-not q. q :- not p.'
```
### Responses

Status: 200 OK 

```
Added data to Solver.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::add failed! No control object."
}
```
```json
{
    "type": "InternalError",
    "msg": "Solver::add failed! Solver has been already started."
}
```

```json
{
    "type": "ClingoError",
    "msg": "InternalError: Call to clingo_control_add() failed, code: Runtime, last: too many messages."
}
```
## Grounding

Method: `GET` 

```url
curl http://localhost:8000/ground
```
### Responses

Status: 200 OK 

```
Grounding.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::ground failed! Solver has been already started."
}
```


## Solving

Method: `GET` 

```url
curl http://localhost:8000/solve
```
### Responses

Status: 200 OK 

```
Solving.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::solve failed! Solving has already started."
}
```

## Poll models

Method: `GET` 

```url
curl http://localhost:8000/model
```

### Responses

Status: 200 OK 

```json
{"Model":[113,10]}
```

```
Done
```

```json
{
    "type": "InternalError",
    "msg": "Solver::model failed! Solving has not yet started."
}
```

## Resume solving

Method: `GET` 

```url
curl http://localhost:8000/resume
```
### Responses

Status: 200 OK 

```
Search is resumed.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::solve failed! Solving has already started."
}
```

## Finish search

Method: `GET` 

```url
curl http://localhost:8000/close
```
### Responses

Status: 200 OK 

```
Solve handle closed.
```

```json
{
    "type": "InternalError",
    "msg": "Solver::close failed! Solving has not yet started."
}
```