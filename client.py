import subprocess
import requests

response = requests.get('http://localhost:8000/')
data = response.text
print(data)

response = requests.get('http://localhost:8000/create')
data = response.text
print(data)

with open('queens.lp', 'rb') as f:
    response = requests.post("http://localhost:8000/add", data=f.read(),
                             headers={
        "X-RapidAPI-Host": "NasaAPIdimasV1.p.rapidapi.com",
        "X-RapidAPI-Key": "4xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "Content-Type": "text/plain; charset=utf-8 "
    }
    )
    data = response.text
    print(data)

response = requests.get('http://localhost:8000/ground')
data = response.text
print(data)
response = requests.get('http://localhost:8000/solve')
data = response.text
print(data)

print("Model1:")
response = requests.get('http://localhost:8000/model')
print(response.text)
print("Model2:")
response = requests.get('http://localhost:8000/model')
print(response.text)
print("Model3:")
response = requests.get('http://localhost:8000/model')
print(response.text)
print("Model4:")
response = requests.get('http://localhost:8000/model')
print(response.text)
print("Model5:")
response = requests.get('http://localhost:8000/model')
print(response.text)
print("Model6:")
response = requests.get('http://localhost:8000/model')
print(response.text)
response = requests.get('http://localhost:8000/close')
print(response.text)
