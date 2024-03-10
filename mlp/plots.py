import matplotlib.pyplot as plt
import numpy as np

# Import data from CSV file
X = np.genfromtxt("./data.csv", delimiter=",")
Z = np.genfromtxt("./training_data.csv", delimiter=",")

# Scatter plot on X
plt.scatter(X[:, 0], X[:, 1])
plt.show()

Y = X[:, 1]
X = X[:, 0]
my_cmap = "viridis"

# # Surface plot on X and Z
# fig = plt.figure()
# ax = plt.axes(projection="3d")
# ax.plot_trisurf(X, Y, Z, cmap=my_cmap, alpha=0.3)
# ax.scatter(X, Y, Z, c=Z, cmap=my_cmap)

X_predict = np.genfromtxt("./data.csv", delimiter=",")
Z_predict = np.genfromtxt("./prediction.csv", delimiter=",")
Z_actual = np.genfromtxt("./training_data.csv", delimiter=",")

Y_predict = X_predict[:, 1]
X_predict = X_predict[:, 0]

fig = plt.figure()
ax = fig.add_subplot(111, projection="3d")
ax.plot_trisurf(X_predict, Y_predict, Z_predict, cmap=my_cmap, alpha=0.3)
ax.scatter(X_predict, Y_predict, Z_predict, c=Z_predict, cmap=my_cmap)
ax.set_title("Predicted")

fig = plt.figure()
ax = fig.add_subplot(111, projection="3d")
ax.plot_trisurf(X_predict, Y_predict, Z_actual, cmap=my_cmap, alpha=0.3)
ax.scatter(X_predict, Y_predict, Z_actual, c=Z_actual, cmap=my_cmap)
ax.set_title("Actual")
plt.show()