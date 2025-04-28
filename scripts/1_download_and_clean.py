import pandas as pd

# 1) Load the raw CSV
df = pd.read_csv(
    "data/lapd_crime_2020_present.csv",
    low_memory=False
)

# 2) Normalize column names
df.columns = [c.strip().replace(" ", "_") for c in df.columns]

# 3) Parse DATE_OCC into datetime
df["DATE_OCC"] = pd.to_datetime(df["DATE_OCC"], errors="coerce")

# 4) Keep only the columns we need and drop rows with missing critical values
keep = ["DR_NO","DATE_OCC","AREA_NAME","LAT","LON","Crm_Cd_Desc"]
df = df[keep].dropna(subset=keep)

# 5) Save the cleaned CSV
df.to_csv("data/clean_crime.csv", index=False)
print("Cleaned data shape:", df.shape)
