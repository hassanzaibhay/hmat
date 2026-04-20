# HMAT Language Specification v0.3
# Standard Library — Including First-Class Data Science / ML / AI
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>
#
# In HMAT, data science, machine learning, and AI are NOT third-party packages.
# They are first-class language primitives in the standard library.
# No pip install. No import aliasing. No API fragmentation.
# One language. One stdlib. Everything works together.

---

## 1. Core Idea

**AI support out of the box. No third-party packages. Ever.**

HMAT's stdlib IS the language. A data scientist, ML engineer, or AI developer
should be able to build production-grade systems using only HMAT and its stdlib.
The following are ALL first-class — not library calls, not optional installs:

- **Tensor operations** — creation, math, linalg, autograd
- **DataFrame operations** — load, query, transform, join, aggregate
- **Classical ML** — train, evaluate, cross-validate, hyperparameter search
- **Neural networks** — define, train, evaluate, save, load, export
- **Statistics** — descriptive, hypothesis tests, distributions, Bayesian
- **Visualization** — all common chart types, interactive plots
- **Data I/O** — all common formats (see priority list below)
- **AI inference** — call any LLM/model, structured output, embeddings
- **AI training** — fine-tune, RLHF, LoRA, full training loops

## 2. Design Principles

| Principle       | Meaning                                                       |
|-----------------|---------------------------------------------------------------|
| First-class     | tensor, frame, nn, learn, stats, plot need no third-party dep |
| Complete AI     | Train, eval, test, deploy — ALL AI ops built-in               |
| Composable      | DS ops integrate with HMAT flows and AI constructs natively   |
| Type-safe       | Tensor shapes tracked at compile time where possible          |
| Gradual rollout | Most-used formats/features first, others added over releases  |
| Performant      | Backed by LLVM + hardware kernels (CUDA/Metal/CPU-SIMD)       |

---

## 3. Module Hierarchy

```
hmat.
├── core/          # Always available — no import
│   ├── types      # Tensor, Frame, Model + primitives
│   ├── ops        # Traits: Comparable, Clone, etc.
│   ├── iter       # Iterators
│   └── convert    # Type conversions
│
├── tensor/        # Tensor operations (auto-imported with tensor keyword)
│   ├── math       # Element-wise math
│   ├── linalg     # Linear algebra
│   ├── random     # Random tensor generation
│   └── grad       # Automatic differentiation
│
├── frame/         # DataFrames (auto-imported with frame keyword)
│   ├── io         # CSV, Parquet, JSON, SQL, Arrow
│   ├── query      # Filter, sort, group, join
│   ├── transform  # Scale, encode, impute, split
│   └── window     # Rolling, expanding windows
│
├── learn/         # Classical ML
│   ├── linear     # Linear and logistic regression
│   ├── tree       # Decision trees, random forests, boosting
│   ├── cluster    # KMeans, DBSCAN, hierarchical
│   ├── reduce     # PCA, t-SNE, UMAP
│   ├── svm        # Support vector machines
│   └── eval       # Metrics and cross-validation
│
├── nn/            # Neural networks
│   ├── layers     # Dense, Conv, Attention, Embedding, RNN
│   ├── optim      # Adam, SGD, AdamW, schedulers
│   ├── loss       # CrossEntropy, MSE, Huber, etc.
│   ├── init       # Weight initialization
│   └── pretrained # Load pretrained weights
│
├── stats/         # Statistics
│   ├── descriptive# Mean, std, percentiles, correlation
│   ├── tests      # t-test, ANOVA, chi-square, KS test
│   ├── dist       # Normal, Binomial, Poisson, etc.
│   └── bayes      # Bayesian inference utilities
│
├── plot/          # Visualization
│   ├── basic      # line, scatter, bar, histogram
│   ├── stats      # boxplot, violin, heatmap, pair plot
│   ├── geo        # Map plots
│   └── interactive# Interactive (outputs to browser)
│
├── data/          # Data utilities
│   ├── loader     # Batched data loading
│   ├── augment    # Data augmentation (images, text, tabular)
│   ├── split      # Train/val/test splitting
│   └── pipeline   # Preprocessing pipelines
│
├── io/            # General IO
│   ├── fs         # Filesystem
│   ├── http       # HTTP client
│   └── formats/   # Data format support (see priority list below)
├── net/           # Networking
├── async/         # Async primitives
├── math/          # General math
├── time/          # Date and time
├── env/           # Environment
└── test/          # Testing framework
```

---

## 4. Supported Data Formats

HMAT supports common data formats out of the box. Most-used formats are
implemented first — others are added in subsequent releases.

### Priority 1 (v0.1 — ship with core stdlib)

| Format     | Read | Write | Notes                              |
|------------|------|-------|------------------------------------|
| CSV        | ✅   | ✅    | delimiter, encoding, header options|
| JSON       | ✅   | ✅    | streaming support for large files  |
| Parquet    | ✅   | ✅    | columnar, compressed, fast         |
| Arrow/IPC  | ✅   | ✅    | zero-copy interop                  |
| Plain text | ✅   | ✅    | line-by-line, whole file           |
| Binary     | ✅   | ✅    | raw bytes                          |

### Priority 2 (v0.2)

| Format     | Read | Write | Notes                              |
|------------|------|-------|------------------------------------|
| Excel      | ✅   | ✅    | .xlsx and .xls, multi-sheet        |
| SQLite     | ✅   | ✅    | embedded SQL database              |
| HDF5       | ✅   | ✅    | hierarchical, used in science/ML   |
| Feather    | ✅   | ✅    | fast pandas/R interop              |
| NDJSON     | ✅   | ✅    | newline-delimited JSON             |
| TOML       | ✅   | ✅    | config files                       |
| YAML       | ✅   | ✅    | config files                       |

### Priority 3 (v0.3+)

| Format     | Notes                                             |
|------------|---------------------------------------------------|
| ORC        | Hadoop columnar format                            |
| Avro       | Schema-based serialization                        |
| MessagePack| Binary JSON alternative                           |
| Protobuf   | Google's binary format                            |
| NetCDF     | Scientific array data                             |
| FITS       | Astronomy data format                             |
| Shapefile  | Geospatial vector data                            |

### Usage

```hmat
# All formats use the same frame.* interface
df = frame.csv("data.csv")
df = frame.json("data.json")
df = frame.parquet("data.parquet")
df = frame.excel("data.xlsx", sheet: "Sheet1")
df = frame.sql("SELECT * FROM users", db: conn)
df = frame.arrow("data.arrow")
df = frame.hdf5("data.h5", key: "/train/features")

# Detect format automatically from extension
df = frame.load("data.csv")      # auto-detected

# Save in any format
df.save("output.parquet")        # auto-detected from extension
df.save_csv("output.csv", sep: ";")
df.save_json("output.json", orient: "records")
df.save_parquet("output.parquet", compression: "snappy")

# Tensors — native formats
t = tensor.load("weights.hmt")   # HMAT tensor format (native)
t = tensor.load("weights.npy")   # NumPy interop
t = tensor.load("weights.pt")    # PyTorch interop
t.save("weights.hmt")
```

---

## 5. Tensor — First-Class Type

`Tensor` is a first-class HMAT type. No import needed.

### 3.1 Creation

```hmat
# 1D
x = tensor[1.0, 2.0, 3.0]          # shape: (3,)

# 2D
y = tensor[[1.0, 2.0], [3.0, 4.0]] # shape: (2,2)

# Constructors
z = zeros(3, 4)                      # shape: (3,4), float
w = ones(100, 784)                   # shape: (100,784)
r = rand(64, 3, 224, 224)           # uniform random, shape: (64,3,224,224)
n = randn(32, 512)                  # normal random
e = eye(4)                          # 4x4 identity matrix
g = arange(0.0, 1.0, step: 0.1)   # [0.0, 0.1, ..., 0.9]
l = linspace(0.0, 1.0, 100)        # 100 evenly spaced points
```

### 3.2 Shape and Type

```hmat
x.shape           # (3,) — tuple of ints
x.rank            # 1
x.size            # 3 (total elements)
x.dtype           # float, int, bool, etc.

# Reshape
y = x.reshape(1, 3)
y = x.view(3, 1)
y = x.flatten()
y = x.unsqueeze(0)   # add dimension at axis 0
y = x.squeeze()      # remove size-1 dimensions
```

### 3.3 Arithmetic — All Broadcast Naturally

```hmat
a = x + y           # element-wise add (broadcast)
b = x - y
c = x * 2.0         # scalar multiply
d = x / y
e = x ^ 2           # element-wise power
f = x @ y           # matrix multiply (2D) / batched matmul (3D+)
g = x.T             # transpose

# Comparisons — return bool tensor
mask = x > 0.5
```

### 3.4 Reduction

```hmat
x.sum()             # scalar
x.sum(axis: 0)     # along axis
x.mean()
x.mean(axis: 1)
x.max()
x.min()
x.argmax()
x.argmin()
x.std()
x.norm()
x.norm(p: 2)
```

### 3.5 Linear Algebra

```hmat
from hmat.tensor.linalg import *

det  = linalg.det(A)
inv  = linalg.inv(A)
rank = linalg.rank(A)
(U, S, V) = linalg.svd(A)
(vals, vecs) = linalg.eig(A)
L = linalg.cholesky(A)
x = linalg.solve(A, b)         # Ax = b
```

### 3.6 Automatic Differentiation

```hmat
# Mark tensors that need gradients
x = rand(10, 5).requires_grad()

# Forward pass
y = (x @ W + b).relu()
loss = y.cross_entropy(targets)

# Backward pass
loss.backward()

# Access gradients
W.grad    # dL/dW
b.grad    # dL/db
```

### 3.7 Device Support

```hmat
x = rand(1000, 1000).cuda()    # GPU (CUDA)
x = rand(1000, 1000).metal()   # GPU (Apple Metal)
x = x.cpu()                    # back to CPU

# Context
using device.cuda():
    result = expensive_computation(x)
```

---

## 5. Frame — First-Class DataFrame

`Frame` is a first-class HMAT type. No import needed.

### 4.1 Creation and Loading

```hmat
# From files
df = frame.csv("data.csv")
df = frame.csv("data.csv", sep: ";", encoding: "utf-8")
df = frame.parquet("data.parquet")
df = frame.json("data.json")
df = frame.excel("data.xlsx", sheet: "Sheet1")
df = frame.sql("SELECT * FROM users", conn: db)
df = frame.arrow("data.arrow")

# Inline
df = frame:
    name:  ["alice", "bob", "charlie"]
    age:   [30, 25, 35]
    score: [0.9, 0.7, 0.85]
    city:  ["NY", "LA", "NY"]

# From tensors
df = frame.from_tensor(X, columns: ["f1", "f2", "f3"])
```

### 4.2 Inspection

```hmat
df.shape          # (rows, cols)
df.columns        # [str]
df.dtypes         # {str: type}
df.head(10)       # first 10 rows
df.tail(10)       # last 10 rows
df.describe()     # statistical summary
df.info()         # column types + null counts
df.nulls()        # null count per column
```

### 4.3 Query — HMAT Pipe Style

HMAT uses `|` as the frame query operator — reads as "where":

```hmat
# Filter
adults  = df | age > 25
ny_high = df | city == "NY" | score >= 0.8

# Sort
sorted  = df | sort(-score)          # descending
sorted  = df | sort(age, -score)     # multi-column

# Select columns
small   = df | select(name, score)
dropped = df | drop(city)

# Limit
top10   = df | take(10)
page2   = df | skip(10) | take(10)

# Chain everything
result = df
    | age > 25
    | city == "NY"
    | sort(-score)
    | select(name, age, score)
    | take(5)
```

### 4.4 Group and Aggregate

```hmat
summary = df.group(by: "city"):
    mean_score:  mean(score)
    total_count: count()
    max_age:     max(age)
    min_age:     min(age)
    sum_score:   sum(score)

# Multiple group keys
df.group(by: ["city", "category"]):
    avg: mean(score)
```

### 4.5 Join

```hmat
merged = df.join(other, on: "user_id")
merged = df.join(other, on: "user_id", how: "left")   # left/right/inner/outer/cross
merged = df.join(other, left: "id", right: "user_id")
```

### 4.6 Transform

```hmat
# Per-column transforms
df.score = df.score.scale()           # min-max normalize to [0,1]
df.score = df.score.standardize()     # z-score normalize
df.age   = df.age.clip(0, 120)        # clip outliers
df.name  = df.name.lower()
df.city  = df.city.encode("onehot")  # one-hot encoding
df.city  = df.city.encode("label")   # label encoding
df.score = df.score.fill_nil(df.score.mean())  # impute nulls

# Apply custom function
df.score = df.score.apply(x => x ^ 2)

# Add computed column
df.bmi = df.weight / (df.height ^ 2)

# Window functions
df.rolling_mean = df.score.rolling(7).mean()
df.cumsum       = df.score.expanding().sum()
```

### 4.7 Export

```hmat
df.save_csv("output.csv")
df.save_parquet("output.parquet")
df.save_json("output.json")
df.to_tensor()                   # Tensor — numeric columns only
df.to_numpy()                    # interop
```

---

## 6. learn — Classical ML

All classical ML algorithms are first-class. No sklearn needed.

### 5.1 Training Pattern

Every learner follows the same pattern:

```hmat
# 1. Configure
model = learn.forest(target: "bought", features: ["age", "income", "city"])

# 2. Train
fit = model.train(df)

# 3. Evaluate
metrics = fit.eval(test_df)
print(metrics.accuracy)
print(metrics.f1)

# 4. Predict
preds = fit.predict(new_df)
probs = fit.predict_proba(new_df)   # probability scores
```

### 5.2 Supervised Learners

```hmat
# Regression
learn.linear(target: "price", features: [...], regularize: "l2", alpha: 0.01)
learn.ridge(target: "price", alpha: 1.0)
learn.lasso(target: "price", alpha: 0.1)
learn.svr(kernel: "rbf", C: 1.0)

# Classification
learn.logistic(target: "label", features: [...])
learn.svm(target: "label", kernel: "rbf", C: 1.0)
learn.knn(target: "label", k: 5, metric: "euclidean")
learn.naive_bayes(target: "label", variant: "gaussian")

# Tree-based (usually best out of the box)
learn.tree(target: "label", max_depth: 10)
learn.forest(target: "label", trees: 100, max_depth: 15)
learn.boost(target: "label", trees: 500, lr: 0.05, max_depth: 6)   # gradient boosting
learn.xgboost(target: "label", trees: 300, lr: 0.1)
```

### 5.3 Unsupervised

```hmat
# Clustering
clusters = learn.cluster.kmeans(k: 5).fit(df)
df.cluster = clusters.labels

clusters = learn.cluster.dbscan(eps: 0.5, min_samples: 10).fit(df)
clusters = learn.cluster.hierarchical(k: 5, linkage: "ward").fit(df)

# Dimensionality reduction
reducer = learn.reduce.pca(components: 50).fit(df)
X_reduced = reducer.transform(df)

reducer = learn.reduce.tsne(components: 2, perplexity: 30).fit(X)
reducer = learn.reduce.umap(components: 2, neighbors: 15).fit(X)
```

### 5.4 Cross-Validation and Evaluation

```hmat
# Cross-validation
cv = learn.cv(model, df, folds: 5, metric: "f1"):
    mean_f1, std_f1 = cv.score

# Hyperparameter search
search = learn.search(model, df):
    trees:     [50, 100, 200, 500]
    max_depth: [5, 10, 15, 20]
    method:    "grid"    # or "random", "bayesian"

best = search.best_model
print(search.results)

# Metrics
metrics = fit.eval(test_df)
metrics.accuracy
metrics.precision
metrics.recall
metrics.f1
metrics.auc_roc
metrics.confusion_matrix
metrics.mse        # regression
metrics.rmse
metrics.mae
metrics.r2
```

### 5.5 Feature Engineering

```hmat
from hmat.learn import features as fe

# Importance
importance = fit.feature_importance()     # works for tree-based models

# Selection
selector = fe.select.variance(threshold: 0.01).fit(df)
selector = fe.select.mutual_info(k: 20, target: "label").fit(df)
selector = fe.select.rfe(model, k: 10).fit(df)

# Polynomial features
poly = fe.polynomial(degree: 2).fit(df)

# Interaction features
inter = fe.interactions(["age", "income"]).fit(df)
```

---

## 7. nn — Neural Networks

Neural networks are first-class HMAT constructs.

### 6.1 Sequential

```hmat
net = nn.seq:
    Dense(784 => 256, relu)
    BatchNorm()
    Dropout(0.3)
    Dense(256 => 128, relu)
    Dropout(0.2)
    Dense(128 => 10, softmax)
```

### 6.2 Functional (Complex Architectures)

```hmat
net = nn.build(x: Tensor) -> Tensor:
    # Residual connection
    h = Dense(256, relu)(x)
    h = BatchNorm()(h)
    h = h + x                    # skip connection
    Dense(10, softmax)(h)

# Multi-input
fusion = nn.build(image: Tensor, text: Tensor) -> Tensor:
    img_feat  = Conv2D(64, 3)(image).relu().MaxPool2D(2)
    txt_feat  = Embedding(50000, 128)(text).LSTM(256)
    combined  = concat(img_feat.flatten(), txt_feat)
    Dense(num_classes, softmax)(combined)
```

### 6.3 Training

```hmat
trained = net.fit(X_train, y_train):
    optimizer: adam(lr: 0.001, weight_decay: 1e-4)
    loss:      cross_entropy
    epochs:    50
    batch:     64
    validate:  (X_val, y_val)
    patience:  5                 # early stopping
    save_best: "model.hm"

# Callbacks
trained = net.fit(X_train, y_train):
    optimizer: adam(lr: 0.001)
    loss:      mse
    epochs:    100
    on_epoch: (epoch, metrics) => print("epoch {epoch}: {metrics.val_loss}")
    on_improve: model => model.save("checkpoint.hm")
```

### 6.4 Available Layers

```hmat
# Dense
Dense(in => out, activation)
Dense(256, relu)                # output size only — input inferred

# Convolutional
Conv1D(filters, kernel, stride: 1, padding: "same")
Conv2D(filters, kernel, stride: 1, padding: "same")
Conv3D(filters, kernel)
ConvTranspose2D(filters, kernel)   # upsampling

# Pooling
MaxPool1D(kernel, stride)
MaxPool2D(kernel, stride)
AvgPool2D(kernel, stride)
GlobalAvgPool()

# Normalization
BatchNorm()
LayerNorm()
GroupNorm(groups: 8)

# Regularization
Dropout(rate)
SpatialDropout2D(rate)

# Recurrent
LSTM(hidden, layers: 1, bidirectional: false)
GRU(hidden, layers: 1)
RNN(hidden, cell: "tanh")

# Attention / Transformers
MultiHeadAttention(heads: 8, dim: 512)
TransformerBlock(heads: 8, dim: 512, ff_dim: 2048, dropout: 0.1)
Transformer(layers: 6, heads: 8, dim: 512, vocab: 50000)

# Embedding
Embedding(vocab_size, embedding_dim)
PositionalEncoding(max_len: 512, dim: 512)

# Activation layers
Activation(relu)
Activation(gelu)
Activation(sigmoid)
Activation(tanh)
Activation(softmax)
```

### 6.5 Activations (First-Class Functions)

```hmat
relu(x)
gelu(x)
silu(x)      # Swish
sigmoid(x)
tanh(x)
softmax(x)
softmax(x, axis: -1)
leaky_relu(x, alpha: 0.01)
elu(x)
```

### 6.6 Loss Functions

```hmat
cross_entropy(pred, target)
binary_cross_entropy(pred, target)
mse(pred, target)               # mean squared error
mae(pred, target)               # mean absolute error
huber(pred, target, delta: 1.0)
hinge(pred, target)
kl_divergence(p, q)
contrastive(anchor, positive, negative, margin: 1.0)
focal(pred, target, gamma: 2.0)
```

### 6.7 Optimizers

```hmat
adam(lr: 0.001, beta1: 0.9, beta2: 0.999, eps: 1e-8)
adamw(lr: 0.001, weight_decay: 0.01)
sgd(lr: 0.01, momentum: 0.9, nesterov: true)
rmsprop(lr: 0.001, alpha: 0.99)
adagrad(lr: 0.01)

# Learning rate schedulers
cosine_annealing(optimizer, T_max: 100)
step_decay(optimizer, step: 10, gamma: 0.1)
warmup_cosine(optimizer, warmup: 1000, total: 10000)
plateau(optimizer, patience: 5, factor: 0.5)
```

### 6.8 Pretrained Models

```hmat
from hmat.nn.pretrained import *

# Load pretrained — downloads on first use, cached locally
resnet   = pretrained.resnet50()
vgg      = pretrained.vgg16()
bert     = pretrained.bert_base()
gpt2     = pretrained.gpt2()
clip     = pretrained.clip()
whisper  = pretrained.whisper_base()

# Fine-tune
finetuned = resnet.finetune(train_df, target: "class"):
    frozen_layers: 40      # freeze first 40 layers
    optimizer: adam(lr: 1e-4)
    epochs: 10
```

### 6.9 Save and Load

```hmat
trained.save("model.hm")
loaded = nn.load("model.hm")
pred = loaded.predict(X_test)

# Export
trained.export.onnx("model.onnx")
trained.export.tflite("model.tflite")
trained.export.torchscript("model.pt")
```

---

## 8. stats — Statistics

```hmat
# Descriptive
stats.mean(data)
stats.median(data)
stats.mode(data)
stats.std(data)
stats.var(data)
stats.skew(data)
stats.kurtosis(data)
stats.quantile(data, 0.95)
stats.iqr(data)
stats.range(data)
stats.corr(x, y)               # Pearson correlation
stats.spearman(x, y)           # Spearman rank correlation
stats.covariance(x, y)
stats.describe(data)           # all at once

# Hypothesis tests
t = stats.t_test(a, b)
print("p={t.p}, t={t.statistic}, significant={t.p < 0.05}")

stats.paired_t_test(before, after)
stats.anova(group1, group2, group3)
stats.chi_square(observed, expected)
stats.ks_test(data, distribution: "normal")
stats.mannwhitney(a, b)
stats.wilcoxon(a, b)

# Distributions — create, sample, compute
normal = stats.normal(mean: 0.0, std: 1.0)
sample = normal.sample(1000)
pdf    = normal.pdf(1.96)
cdf    = normal.cdf(1.96)
ppf    = normal.ppf(0.975)     # percent point function (inverse CDF)

stats.binomial(n: 10, p: 0.5)
stats.poisson(lam: 3.0)
stats.uniform(low: 0.0, high: 1.0)
stats.exponential(rate: 1.0)
stats.beta(alpha: 2.0, beta: 5.0)
stats.gamma(shape: 2.0, scale: 1.0)

# Bayesian
from hmat.stats.bayes import *
posterior = bayes.update(prior: normal, likelihood: binomial, data: observations)
```

---

## 9. plot — Visualization

All plots render to screen by default. Save with `.save("path")`.

```hmat
# Line
plot.line(x, y)
plot.line(x, y, label: "train loss", color: "blue", linewidth: 2)
plot.line(epochs, [train_loss, val_loss], labels: ["train", "val"])

# Scatter
plot.scatter(x, y)
plot.scatter(df.age, df.score, color: df.city, size: df.weight)

# Bar
plot.bar(categories, values)
plot.bar(df.city, df.count, horizontal: true)

# Histogram
plot.histogram(data)
plot.histogram(data, bins: 50, density: true)

# Box and violin
plot.boxplot(df.score, group: df.city)
plot.violin(df.score, group: df.category)

# Heatmap
plot.heatmap(correlation_matrix, labels: column_names)
plot.heatmap(confusion_matrix, title: "Confusion Matrix")

# Statistical
plot.pair(df, hue: "class")         # pair plot
plot.residuals(fit)                  # residual plot
plot.roc_curve(fit, test_df)         # ROC curve
plot.learning_curve(model, df)       # learning curve
plot.feature_importance(fit)         # bar chart of importance

# 3D
plot.scatter3d(x, y, z, color: labels)
plot.surface(X, Y, Z)

# Subplots
fig = plot.figure(rows: 2, cols: 2):
    [0, 0] => plot.line(x, train_loss, title: "Train Loss")
    [0, 1] => plot.line(x, val_loss, title: "Val Loss")
    [1, 0] => plot.histogram(errors, title: "Error Distribution")
    [1, 1] => plot.roc_curve(fit, test_df, title: "ROC")

# Export
plot.show()
plot.save("results.png", dpi: 300)
plot.save("results.svg")
plot.save("results.html")   # interactive
```

---

## 10. data — Data Loading and Preprocessing

```hmat
from hmat.data import *

# Dataset and loader
dataset = data.Dataset(X, y)
loader  = data.Loader(dataset, batch: 32, shuffle: true, workers: 4)

for (X_batch, y_batch) in loader:
    loss = net.train_step(X_batch, y_batch)

# Train/val/test split
(train, val, test) = data.split(df, ratios: [0.7, 0.15, 0.15], stratify: "label")
(X_train, X_val, X_test), (y_train, y_val, y_test) = data.split(X, y, ratios: [0.8, 0.1, 0.1])

# Data augmentation — images
aug = data.augment.image:
    RandomFlip(horizontal: true)
    RandomRotate(degrees: 15)
    RandomCrop(size: 224)
    ColorJitter(brightness: 0.2, contrast: 0.2)
    Normalize(mean: [0.485, 0.456, 0.406], std: [0.229, 0.224, 0.225])

# Data augmentation — text
aug = data.augment.text:
    RandomSynonymSwap(p: 0.1)
    RandomDeletion(p: 0.05)
    BackTranslation(lang: "de")

# Preprocessing pipeline — composable with HMAT flows
flow preprocess_images:
    load_image => resize(224) => aug => normalize => to_tensor

flow preprocess_tabular:
    load_csv => fill_nil => scale => encode_categoricals => to_tensor
```

---

## 11. DS + AI Integration (Unique to HMAT)

Because AI is a first-class HMAT primitive, DS and AI compose naturally:

```hmat
ai llm = model("anthropic/claude-3-5-sonnet")

# AI-assisted feature engineering
flow ai_features:
    df.description => llm.embed => tensor => pca(50) => df.text_features

# AI-augmented labeling
unlabeled = frame.csv("unlabeled.csv")
for row in unlabeled:
    label = llm.classify(row.text, labels: known_classes) or "unknown"
    row.label = label

# AI model explanation
explanation = llm.think(f"""
    This ML model was trained on {df.describe()}.
    It made prediction: {prediction} for input: {input_row}.
    Explain why in plain English.
""")

# Hybrid pipeline — classical ML + LLM
flow hybrid_classifier:
    input
    => extract_features          # classical feature engineering
    => learn.forest.predict      # fast classical ML prediction
    => llm.explain               # AI explains the prediction
    => output
```

---

## 12. Core Standard Library (Non-DS)

### 11.1 Core (Auto-Imported)

```hmat
print(value)
eprint(value)         # stderr
input(prompt) -> str
panic(msg)
assert(cond)
assert(cond, msg)
type_of(value) -> str
size_of[T]() -> int
```

### 11.2 io

```hmat
from hmat.io import fs

fs.read("file.txt") or ""
fs.write("file.txt", content)
fs.append("file.txt", content)
fs.exists("file.txt")
fs.delete("file.txt")
fs.list("dir/")
```

### 11.3 math

```hmat
from hmat.math import *

sqrt(x), pow(x, n), abs(x)
floor(x), ceil(x), round(x)
sin(x), cos(x), tan(x)
log(x), log2(x), ln(x)
min(a, b), max(a, b), clamp(x, lo, hi)
PI, E, INF, NAN
```

### 11.4 async

```hmat
from hmat.async import join, race, sleep, timeout

await sleep(1.5)
result = await timeout(5.0, slow_op()) or default
(a, b) = join(task_a(), task_b()) or (nil, nil)
first  = await race(fast(), slow())
```

### 11.5 test

```hmat
@test
test_add():
    assert_eq(add(2, 3), 5)
    assert_ne(add(2, 3), 6)

@test
test_divide():
    assert_ok(divide(10, 2))
    assert_fail(divide(10, 0))
    assert_approx(divide(1, 3) or 0, 0.333, tolerance: 0.001)

@bench
bench_sort():
    data = rand(10_000)
    data.sort()
```

---

## 13. Naming Conventions

| Element         | Convention            | Example                    |
|-----------------|-----------------------|----------------------------|
| Variables       | snake_case            | train_loss, batch_size     |
| Functions       | snake_case            | load_data, fit_model       |
| Shapes (structs)| PascalCase            | TrainConfig, ModelMetrics  |
| Types (enums)   | PascalCase            | Activation, LossFunction   |
| Constants       | SCREAMING_SNAKE_CASE  | MAX_EPOCHS, LEARNING_RATE  |
| Modules         | snake_case            | hmat.nn.layers             |
