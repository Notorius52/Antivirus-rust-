import numpy as np
from sklearn.ensemble import RandomForestClassifier
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType
import os

# Crea la cartella models se non esiste
if not os.path.exists('models'):
    os.makedirs('models')

X = np.array([
    [64, 10.0], [128, 8.0], [512, 15.0], [1500, 20.0], [1000, 12.0],
    [1500, 0.1], [1480, 0.2], [1500, 0.05], [64, 0.01], [128, 0.02]
], dtype=np.float32)

y = np.array([0, 0, 0, 0, 0, 1, 1, 1, 1, 1])

clf = RandomForestClassifier(n_estimators=50)
clf.fit(X, y)

initial_type = [('float_input', FloatTensorType([None, 2]))]
onx = convert_sklearn(clf, initial_types=initial_type)

with open("models/antivirus_brain.onnx", "wb") as f:
    f.write(onx.SerializeToString())

print("---")
print("Cervello IA v1.2 generato in models/antivirus_brain.onnx")
print("---")
