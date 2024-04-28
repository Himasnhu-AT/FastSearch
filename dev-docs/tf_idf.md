# TF_IDF

### Term Frequency - Inverse Document Frequency

TF-IDF is a numerical statistic that is intended to reflect how important a word is to a document in a collection or corpus. It is often used as a weighting factor in searches of information retrieval, text mining, and user modeling. The TF-IDF value increases proportionally to the number of times a word appears in the document and is offset by the number of documents in the corpus that contain the word, which helps to adjust for the fact that some words appear more frequently in general.

The formula for calculating the TF-IDF of term t is:

```
TF-IDF(t) = TF(t) * IDF(t)
```

Where:

- `TF(t)` is the term frequency of term t in a document
- `IDF(t)` is the inverse document frequency of term t across a set of documents

The term frequency of term t in a document is calculated as:

```
TF(t) = (Number of times term t appears in a document) / (Total number of terms in the document)
```

The inverse document frequency of term t is calculated as:

```
IDF(t) = log_e(Total number of documents / Number of documents with term t in it)
```

The TF-IDF value of a term is high when it appears many times in a small number of documents. Conversely, the value is low when the term appears fewer times in a large number of documents.

### Example

Consider a document containing 100 words wherein the word 'cat' appears 3 times. The term frequency (i.e., TF) for 'cat' is then (3 / 100) = 0.03. Now, assume we have 10 million documents and the word 'cat' appears in 1000 of these. Then, the inverse document frequency (i.e., IDF) is calculated as log(10,000,000 / 1,000) = 4. Thus, the TF-IDF value is 0.03 \* 4 = 0.12.

### References

- [Wikipedia](https://en.wikipedia.org/wiki/Tf%E2%80%93idf)
