import pandas as pd
import json
import os

os.makedirs("data/tracks", exist_ok=True)

df = pd.read_csv("data/test.csv", nrows=300)
# df = pd.read_csv("data/train.csv", nrows=1000)

file_count = 1
for idx, row in df.iterrows():
    polyline_str = row['POLYLINE']
    
    if not polyline_str or polyline_str == "[]":
        continue
        
    coordinates = json.loads(polyline_str)
    
    with open(f"data/tracks/{file_count}.json", "w") as f:
        json.dump(coordinates, f)
        
    file_count += 1
    if file_count > 300:
        break

print(f"Successfully generated {file_count - 1} track files.")
