import pandas as pd
import matplotlib.pyplot as plt

# 1. Read the degree-counts CSV that Rust wrote
df = pd.read_csv("report/degree_counts.csv", names=["degree","count"], header=0)

# 2. Make a log–log plot
plt.figure()
plt.loglog(df["degree"], df["count"], marker='o', linestyle='None')
plt.xlabel("Degree")
plt.ylabel("Count")
plt.title("Log–Log Degree Distribution")

# 3. Save the figure
plt.savefig("report/degree_loglog.png")
print("Plot saved to report/degree_loglog.png")
