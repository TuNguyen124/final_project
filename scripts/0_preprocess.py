import pandas as pd

# 1) Load the cleaned crime CSV
df = pd.read_csv("data/clean_crime.csv", parse_dates=["DATE_OCC"])

# 2) Extract calendar DAY and AREA_NAME
df["DAY"] = df["DATE_OCC"].dt.date
pairs = df[["DAY", "AREA_NAME"]]

# 3) Drop duplicates so each (day, area) is unique
unique = pairs.drop_duplicates()

# 4) Save the reduced dataset
unique.to_csv("data/day_area.csv", index=False)

print(f"Rows before: {len(pairs)}, after dedupe: {len(unique)}")
