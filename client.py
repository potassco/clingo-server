import requests

response = requests.get('http://localhost:8000/')
data = response.text
print(data)


with open('queens.lp', 'rb') as f:
    response = requests.post("http://localhost:8000/upload", data=f.read(),
        headers={
        "X-RapidAPI-Host": "NasaAPIdimasV1.p.rapidapi.com",
        "X-RapidAPI-Key": "4xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "Content-Type": "text/plain; charset=utf-8 "
        }
    )
    data = response.text
    print(data)

response = requests.get('http://localhost:8000/retrieve')
data = response.text
print(data)
