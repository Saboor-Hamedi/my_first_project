[Reciprocal Rank Fusion (RRF)](https://www.scaler.com/topics/reciprocal-rank-fusion/) calculates a unified score for each document by summing the reciprocal of its rank position across multiple retrieval lists. [1, 2]
## The RRF Formula
$$\text{RRF\_score}(d) = \sum_{m \in M} \frac{1}{k + \text{rank}_m(d)}$$

* d: A specific document.
* M: The set of ranked result lists (e.g., keyword search and vector/semantic search).
* $\text{rank}_m(d)$: The position/rank of document d in the m-th list (typically starting at 1 for the top result).
* k: A constant smoothing parameter (commonly set to 60 as proposed in the original paper) used to dampen the influence of high ranks. [1, 2, 3]

------------------------------
## Step-by-Step Calculation Example
Assume two distinct search lists with a constant k = 60:

* List 1 (Keyword Search): ["DocA", "DocB", "DocC"]
* List 2 (Semantic Search): ["DocB", "DocA", "DocD"]

## 1. Identify Ranks for Each Document

* DocA: Rank 1 in List 1, Rank 2 in List 2
* DocB: Rank 2 in List 1, Rank 1 in List 2
* DocC: Rank 3 in List 1, Not in List 2 (ignored for List 2)
* DocD: Not in List 1, Rank 3 in List 2 (ignored for List 1)

## 2. Compute Individual Scores

* Score for DocA:
$$\frac{1}{60 + 1} + \frac{1}{60 + 2} = \frac{1}{61} + \frac{1}{62} \approx 0.01639 + 0.01613 = 0.03252$$
* Score for DocB:
$$\frac{1}{60 + 2} + \frac{1}{60 + 1} = \frac{1}{62} + \frac{1}{61} \approx 0.01613 + 0.01639 = 0.03252$$
* Score for DocC:
$$\frac{1}{60 + 3} = \frac{1}{63} \approx 0.01587$$
* Score for DocD:
$$\frac{1}{60 + 3} = \frac{1}{63} \approx 0.01587$$

## 3. Final Ranking
Sort the documents by their final aggregated RRF score in descending order (DocA and DocB tie for the top spots, followed by DocC and DocD). [2, 3]
If you'd like, I can show you:

* A Python code snippet implementing this exact math
* How to apply custom weights to specific search engines (like weighting keyword vs. semantic search differently)

Let me know what you need next!

[1] [https://www.paradedb.com](https://www.paradedb.com/learn/search-concepts/reciprocal-rank-fusion)
[2] [https://www.scaler.com](https://www.scaler.com/topics/reciprocal-rank-fusion/)
[3] [https://shivamagarwal7.medium.com](https://shivamagarwal7.medium.com/search-reciprocal-rank-fusion-9735dcd1906d)
