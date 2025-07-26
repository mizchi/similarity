Source: https://arxiv.org/abs/2404.08817

---

# Revisiting Code Similarity Evaluation with Abstract Syntax Tree Edit Distance

Yewei Song
Cedric Lothritz Daniel Tang Tegawendé F. Bissyandé Jacques Klein University of Luxembourg
6 Rue Richard Coudenhove-Kalergi
Luxembourg

###### Abstract

This paper revisits recent code similarity evaluation metrics, particularly focusing on the application of Abstract Syntax Tree (AST) editing distance in diverse programming languages.
In particular, we explore the usefulness of these metrics and compare them to traditional sequence similarity metrics.
Our experiments showcase the effectiveness of AST editing distance in capturing intricate code structures, revealing a high correlation with established metrics. Furthermore, we explore the strengths and weaknesses of AST editing distance and prompt-based GPT similarity scores in comparison to BLEU score, execution match, and Jaccard Similarity.
We propose, optimize, and publish an adaptable metric that demonstrates effectiveness across all tested languages, representing an enhanced version of Tree Similarity of Edit Distance (TSED).

Revisiting Code Similarity Evaluation with Abstract Syntax Tree Edit Distance

Yewei Song Cedric Lothritz Daniel Tang Tegawendé F. Bissyandé Jacques Klein University of Luxembourg 6 Rue Richard Coudenhove-Kalergi Luxembourg

## 1 Introduction and Related Work

In the fields of natural language processing and software engineering, code generation tasks are gaining more and more attention. Assessing the quality of generated code is now critically important, but we still lack evaluation methods other than traditional statistical sequence evaluation methods. Widely used semantic evaluation metrics like BLEU score and Jaccard similarity rely on statistical characteristics, overlooking the intricate grammatical structures and logical relationships inherent in complex programming languages.

However, recent developments in the NLP field paved the way for novel evaluation metrics which we explore in this study. For one, the staggering number of powerful large language models (LLMs) such as GPT-3.5/4 Achiam et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib1)) revolutionized the NLP landscape and led to noteworthy advancements in the realm of code review and evaluation Wang et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib16)); Tang et al. ( [2024](https://arxiv.org/html/2404.08817v1#bib.bib13)). Another recent study introduced the novel TSED metric and used it to evaluate text-to-SQL tasks Song et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib12)). For this study, we take advantage of these developments to (1) prompt the GPT-4 model to generate similarity scores for code, and (2) expand on the TSED metric.

We utilize these two different metrics (GPT and TSED) to evaluate the structural similarity of different programming languages and how they relate to execution matches. Furthermore, we address how these metrics are correlated to semantic similarity metrics like the BLEU score. Finally, we investigate some limitations of these metrics by delving into the impact of TSED’s penalty weight of tree operations on evaluation accuracy and exploring the stability of outputs from the GPT LLMs.

As a result, we have these 3 contributions from this research: (a) we propose and publish a new tool for 48 programming languages 1 1 1 https://anonymous.4open.science/r/TSEDwP-7208/README.md, (b) we discuss 2 recent evaluation metrics and 2 traditional metrics and compare them via correlation coefficient, recall to execution match, (c) we discuss the unstable nature of GPT similarity scoring and the ways to optimize TSED.

## 2 Approaches

### 2.1 TSED on Programming Languages

Applying the TSED evaluation method, initially designed for SQL analysis, we have undergone modifications to extend its applicability to various programming languages. The fundamental TSED approach, illustrated in Figure [1](https://arxiv.org/html/2404.08817v1#S2.F1), encompasses AST parsing, AST Editing Distance Calculation, and normalization, closely resembling the methodology outlined in the original paper. However, we have made modifications to both the AST parsing and normalization.

![Refer to caption]() Figure 1: Pipeline of TSED Code Evaluation Metric Code Parsing: Parsing in the domain of programming languages involves parsing raw code text into its associated AST. This parsing underscores the complexity of interpreting various programming constructs and converting them into a structured grammar tree representation.

We use tree-sitter 2 2 2 https://tree-sitter.github.io/tree-sitter/ as our AST parser which is based on GLR, a powerful parsing algorithm commonly found in the literature Latif et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib6)); Tomita ( [1991](https://arxiv.org/html/2404.08817v1#bib.bib15)); Clem and Thomson ( [2021](https://arxiv.org/html/2404.08817v1#bib.bib4)).

Tree Distance Computation: For calculating tree edit distance as Δ Δ \\Delta roman_Δ, we utilize the same function as outlined in the TSED paper, which is APTED algorithm Pawlik and Augsten ( [2015](https://arxiv.org/html/2404.08817v1#bib.bib10), [2016](https://arxiv.org/html/2404.08817v1#bib.bib11)).
Considering G 1 subscript 𝐺 1 G\_{1} italic_G start_POSTSUBSCRIPT 1 end_POSTSUBSCRIPT as predicted code’s AST and G 2 subscript 𝐺 2 G\_{2} italic_G start_POSTSUBSCRIPT 2 end_POSTSUBSCRIPT are AST from ground-truth:

| --- | --- | --- | --- |
| | Δ ⁢ ( G 1, G 2) = min o ⁢ p ⁢ s ⁢ ∑ i = 1 n w ⁢ ( o ⁢ p i) Δ subscript 𝐺 1 subscript 𝐺 2 subscript 𝑜 𝑝 𝑠 superscript subscript 𝑖 1 𝑛 𝑤 𝑜 subscript 𝑝 𝑖 \\Delta(G\_{1},G\_{2})=\\min\_{ops}\\sum\_{i=1}^{n}w(op\_{i}) roman_Δ ( italic_G start_POSTSUBSCRIPT 1 end_POSTSUBSCRIPT , italic_G start_POSTSUBSCRIPT 2 end_POSTSUBSCRIPT ) = roman_min start_POSTSUBSCRIPT italic_o italic_p italic_s end_POSTSUBSCRIPT ∑ start_POSTSUBSCRIPT italic_i = 1 end_POSTSUBSCRIPT start_POSTSUPERSCRIPT italic_n end_POSTSUPERSCRIPT italic_w ( italic_o italic_p start_POSTSUBSCRIPT italic_i end_POSTSUBSCRIPT ) | | (1) |

Here, o ⁢ p ⁢ s 𝑜 𝑝 𝑠 ops italic_o italic_p italic_s is a sequence of edit operations transforming G 1 subscript 𝐺 1 G\_{1} italic_G start_POSTSUBSCRIPT 1 end_POSTSUBSCRIPT into G 2 subscript 𝐺 2 G\_{2} italic_G start_POSTSUBSCRIPT 2 end_POSTSUBSCRIPT, with w ⁢ ( o ⁢ p i) 𝑤 𝑜 subscript 𝑝 𝑖 w(op\_{i}) italic_w ( italic_o italic_p start_POSTSUBSCRIPT italic_i end_POSTSUBSCRIPT ) as the cost for the i t ⁢ h superscript 𝑖 𝑡 ℎ i^{th} italic_i start_POSTSUPERSCRIPT italic_t italic_h end_POSTSUPERSCRIPT operation.

Normalization: Normalization of tree edit distances accounts for the complexity of the code by considering the maximum number of nodes between two trees, and we add a ramp function to avoid some extreme situations:

| --- | --- | --- | --- |
| | T ⁢ S ⁢ E ⁢ D = max ⁡ { 1 − δ M ⁢ a ⁢ x ⁢ N ⁢ o ⁢ d ⁢ e ⁢ s ⁢ ( G 1, G 2), 0 } 𝑇 𝑆 𝐸 𝐷 1 𝛿 𝑀 𝑎 𝑥 𝑁 𝑜 𝑑 𝑒 𝑠 subscript 𝐺 1 subscript 𝐺 2 0 TSED=\\max\\{1-\\frac{\\delta}{MaxNodes(G\_{1},G\_{2})},0\\} italic*T italic_S italic_E italic_D = roman_max { 1 - divide start_ARG italic*δ end_ARG start_ARG italic_M italic_a italic_x italic_N italic_o italic_d italic_e italic_s ( italic_G start_POSTSUBSCRIPT 1 end_POSTSUBSCRIPT , italic_G start_POSTSUBSCRIPT 2 end_POSTSUBSCRIPT ) end_ARG , 0 } | | (2) |

This provides a metric for structural similarity comparison of programming code, enabling a nuanced analysis beyond mere syntactic comparison.

### 2.2 GPT Structure Similarity

Between 2020 and 2023, OpenAI introduced the GPT-3/3.5 and GPT-4 models, showcasing remarkable reasoning capabilities and achieving state-of-the-art performance across numerous tasks Brown et al. ( [2020](https://arxiv.org/html/2404.08817v1#bib.bib3)). Our approach involves utilizing prompts to elicit the model’s output regarding the structural similarity between two code segments, resulting in a score on a scale from 0 to 1. A score of 1 indicates identical structures, while 0 signifies complete dissimilarity. Despite its effectiveness, this metric operates as a black box, leaving us unaware of the specific calculations performed by GPT or whether it consistently employs the same metric. From various research papers, we’ve observed that these LLMs tend to produce more unstable results with each iteration Tian et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib14)); Liu et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib8)).

## 3 Research Questions and Targets

RQ1: Can TSED be used in more programming languages? We investigate the adaptability of AST Edit Distance which is a generalized version of TSED, exploring its effectiveness in languages like Python and Java to assess its applicability for code similarity analysis.
RQ2: How are TSED and GPT similarity correlated to semantic similarity and execution match? We assess the correlation between these different metrics to understand their respective contributions in evaluating code similarity across multiple programming languages.
RQ3: What are the limits of these metrics? We assess the stability of GPT-based similarity output and analyze how parameters, particularly operation weights (delete, insert, rename), influence TSED.

## 4 Experiments

### 4.1 General Setup

In this study, our primary objective is to apply the theoretical framework to a diverse range of programming languages. To achieve this, we aim to identify executable datasets and evaluate them using predefined metrics. The experimental setup comprises two key tasks: firstly, expanding the application of TSED and GPT similarity to additional programming languages, followed by exploring the correlation between these metrics. Subsequently, we seek to assess the stability of GPT scoring and examine the impact of various parameters on the TSED metric. This structured approach allows us to comprehensively investigate the adaptability, correlations, and stability of the chosen metrics across a spectrum of programming languages.

### 4.2 Evaluation Metrics

- • BLEU Score is calculated as the geometric mean of the modified precision scores for various n-gram lengths, providing a concise and standardized similarity measurement between the generated and reference text Papineni et al. ( [2002](https://arxiv.org/html/2404.08817v1#bib.bib9)).
- • Jaccard Similarity is a measure of similarity between two sets and is calculated by dividing the size of the intersection of the sets by the size of their union, offering a quantitative assessment of the degree of overlap between the sets’ elements.
- • Execution Match Execution Match pertains to the consistency in execution outcomes between generated code and its corresponding ground truth, evaluating the equivalence in practical functionality. 1 in Execution match means they have the same execution results, and 0 means different.
- • GPT Similarity mentioned in the Section [2.2](https://arxiv.org/html/2404.08817v1#S2.SS2)
- • TSED mentioned in the Section [2.1](https://arxiv.org/html/2404.08817v1#S2.SS1).

### 4.3 Datasets

Although the execution match metric is infrequently employed in programming code-related datasets, its prominence has increased in recent years. Our comparative analysis involved assessing datasets from various papers, considering factors such as dataset sizes, programming languages, and executables. As highlighted in Table [1](https://arxiv.org/html/2404.08817v1#S4.T1), the MBXP dataset encompasses 13 different languages, serving as a function-level benchmark that effectively evaluates programming paragraphs. However, the MBXP dataset includes ground-truth solutions for only 7 languages, with C# omitted due to compilation issues. Additionally, we consider the CoderEval dataset to facilitate a comparison between Python and Java code generation, leveraging its longer test samples, results are in the appendix.

| Table 1: Widely-used code generation benchmarks, selected from GitHub            | Benchmark    | Language  | Samples | Executable |
| -------------------------------------------------------------------------------- | ------------ | --------- | ------- | ----------- |
| CoNaLA Yin et al. ( [2018](https://arxiv.org/html/2404.08817v1#bib.bib18))       | Python       | 500       | No      |
| Concode Iyer et al. ( [2018](https://arxiv.org/html/2404.08817v1#bib.bib5))      | Java         | 2000      | No      |
| MBXP Athiwaratkun et al. ( [2022](https://arxiv.org/html/2404.08817v1#bib.bib2)) | Multilingual | 974       | Yes     |
| InterCode Yang et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib17))   | Bash, SQL    | 200, 1034 | Yes     |
| CoderEval Yu et al. ( [2024](https://arxiv.org/html/2404.08817v1#bib.bib19))     | Python, Java | 230       | Yes     |
| RepoEval Liao et al. ( [2023](https://arxiv.org/html/2404.08817v1#bib.bib7))     | Python       | 383       | No      |

In the Bash-Shell scenarios, we reproduce results and conduct a comparative analysis using the InterCode dataset. Notably, we identify the SPIDER dataset within InterCode and establish it as a baseline. SPIDER, previously evaluated in comparison to the TSED paper, is a substantial human-labeled dataset for the text-to-SQL task. This dataset encompasses databases with intricate join solutions across diverse domains Yu et al. ( [2018](https://arxiv.org/html/2404.08817v1#bib.bib20)).

## 5 Results

### 5.1 Similarity Results

| Table 2: Evaluation Metrics comparison for 6 languages on MBXP dataset, prediction generated by GPT-3.5-Turbo model, ground truth from dataset | Languages | TSED   | BLEU   | Jaccard Sim | GPT-4  | Execution |
| ---------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ------ | ------ | ----------- | ------ | --------- |
| Java                                                                                                                                           | 0.3746    | 0.2041 | 0.2733 | 0.8143      | 0.6550 |
| Python                                                                                                                                         | 0.1888    | 0.0843 | 0.2000 | 0.6751      | 0.6842 |
| JavaScript                                                                                                                                     | 0.2037    | 0.0846 | 0.2037 | 0.6763      | 0.6811 |
| Typescript                                                                                                                                     | 0.1360    | 0.0637 | 0.1397 | 0.5313      | 0.6642 |
| Ruby                                                                                                                                           | 0.1727    | 0.0438 | 0.1810 | 0.7067      | 0.6428 |
| Kotlin                                                                                                                                         | 0.3412    | 0.1847 | 0.3109 | 0.7073      | 0.5569 |

| Table 3: Execution Match F1 score & Accuracy for each thresholding metrics | Languages | TSED   | GPT    | BLEU | Jaccard |        |      |        |        |      |        |        |     |
| -------------------------------------------------------------------------- | --------- | ------ | ------ | ---- | ------- | ------ | ---- | ------ | ------ | ---- | ------ | ------ | --- |
| Python                                                                     | 0.23      | 0.5650 | 0.6057 | 0.83 | 0.6403  | 0.6735 | 0.07 | 0.5719 | 0.6150 | 0.19 | 0.5907 | 0.6253 |
| Java                                                                       | 0.10      | 0.5108 | 0.6499 | 0.56 | 0.5693  | 0.6396 | 0.03 | 0.5184 | 0.5755 | 0.16 | 0.5612 | 0.6018 |
| JavaScript                                                                 | 0.12      | 0.5494 | 0.6002 | 0.69 | 0.5924  | 0.6205 | 0.02 | 0.4964 | 0.5267 | 0.12 | 0.5245 | 0.5885 |
| Typescript                                                                 | 0.07      | 0.5367 | 0.5822 | 0.51 | 0.5521  | 0.5708 | 0.01 | 0.4987 | 0.5553 | 0.08 | 0.5284 | 0.5708 |
| Ruby                                                                       | 0.13      | 0.5045 | 0.5306 | 0.54 | 0.6051  | 0.6811 | 0.01 | 0.4375 | 0.4490 | 0.12 | 0.5142 | 0.5612 |
| Kotlin                                                                     | 0.28      | 0.6834 | 0.6823 | 0.8  | 0.6681  | 0.6721 | 0.1  | 0.6441 | 0.6457 | 0.22 | 0.6387 | 0.6533 |

As we analyze the results presented in Table [2](https://arxiv.org/html/2404.08817v1#S5.T2), our experiment demonstrates the effective performance of TSED and GPT similarity in evaluating the MBXP dataset across all 6 programming languages. No instances of parsing or scoring generation failures were observed, confirming the robustness of these metrics across languages.

![Refer to caption]() Figure 2: MBXP dataset, Pearson Correlation Heatmap between evaluation-metrics on GPT-3.5 Moreover, TSED shows a commendable correlation ranging from 0.6 to 0.8 with BLEU score and Jaccard similarity, as illustrated in Figure [2](https://arxiv.org/html/2404.08817v1#S5.F2). Additionally, TSED exhibits a strong correlation with GPT similarity, especially in Java and Python during the CoderEval test, as depicted in Figure [3](https://arxiv.org/html/2404.08817v1#S5.F3), underscoring its sensitivity to code structure.
We employ thresholding to establish a prediction-to-execution match. If the metric value exceeds the threshold T 𝑇 T italic_T, we assign the prediction as 1; otherwise, it is set to 0. The optimal threshold values are determined through enumeration to achieve the best match results. Based on their F1/Accuracy match to the Execution match, both TSED and GPT similarity exhibit higher accuracy compared to semantic metrics in Table [3](https://arxiv.org/html/2404.08817v1#S5.T3). Notably, GPT similarity demonstrates a slightly superior F1 score and TSED gives good results on accuracy.

![Refer to caption]() Figure 3: CoderEval Pearson Correlation Heatmap between evaluation-metrics/models/languages ### 5.2 Stability of GPT Scoring

To understand how unstable GPT scoring is, we execute the GPT-4 Similarity scoring five times on identical prediction sets, we establish the initial result as a baseline to assess differences through statistical indicators such as Mean Squared Error (MSE) or Mean Absolute Error (MAE) in comparison to the first scoring. Table [4](https://arxiv.org/html/2404.08817v1#S5.T4) demonstrates that GPT scoring exhibits limited stability in the context of code similarity evaluation.

| Table 4: Unstable nature of GPT-4 scoring output | Metrics | 1st    | 2nd    | 3rd    | 4th |
| ------------------------------------------------ | ------- | ------ | ------ | ------ | --- |
| Mean Squared Error                               | 0.0581  | 0.0583 | 0.0527 | 0.0628 |
| Mean Absolute Error                              | 0.1902  | 0.1940 | 0.1825 | 0.1996 |

### 5.3 Parameter optimization of TSED

We can configure the penalty weight of 3 operations in tree distance computing: Delete, Insert, and Rename. Figure [4](https://arxiv.org/html/2404.08817v1#S5.F4) which is from a test for the MBXP/Java dataset shows is ‘Insert’ has a sweet spot of 0.8. ’Delete’ and ’Rename’ operations just keep them in 1.0 penalty weight as the best choice. But we need to keep in mind it can be different in other programming languages.

![Refer to caption]() Figure 4: Change each of penalty weight influence correlation to GPT structure similarity score ## 6 Conclusion

In this paper, we applied TSED to more programming languages, compared GPT similarity and TSED to semantic metrics, and checked representation to execution match. Then we discuss limitations about the stability of GPT scoring and the penalty parameters of TSED.

## Limitations

While our study provides valuable insights into code similarity assessment using TSED and GPT-based metrics, it is essential to acknowledge certain limitations. Firstly, the generalizability of our findings may be influenced by the specific datasets and programming languages employed in our analysis. Additionally, the stability of GPT-based similarity metrics, as highlighted in our results, poses a limitation in terms of consistent and reliable code assessments. Furthermore, variations in the interpretation and definition of similarity metrics across different studies may introduce inherent biases. Lastly, the effectiveness of TSED metrics may be contingent upon the quality of the employed parsers and the fine-tuning of penalty parameters. These limitations underscore the need for caution when extrapolating our results to diverse contexts and emphasize the necessity for further research to address these challenges.

## Ethics Statement

Our research adheres to ethical standards, prioritizing integrity and respect for all involved parties. We ensured data privacy, obtained informed consent where applicable, and maintained transparency in our methodologies. The study was conducted with the utmost consideration for ethical guidelines and the welfare of participants, upholding the principles of fairness, accountability, and academic integrity throughout the research process.

## References

- Achiam et al. (2023)
  Josh Achiam, Steven Adler, Sandhini Agarwal, Lama Ahmad, Ilge Akkaya, Florencia Leoni Aleman, Diogo Almeida, Janko Altenschmidt, Sam Altman, Shyamal Anadkat, et al. 2023.

Gpt-4 technical report.

_arXiv preprint arXiv:2303.08774_.

- Athiwaratkun et al. (2022)
  Ben Athiwaratkun, Sanjay Krishna Gouda, Zijian Wang, Xiaopeng Li, Yuchen Tian, Ming Tan, Wasi Uddin Ahmad, Shiqi Wang, Qing Sun, Mingyue Shang, et al. 2022.

Multi-lingual evaluation of code generation models.

_arXiv preprint arXiv:2210.14868_.

- Brown et al. (2020)
  Tom B. Brown, Benjamin Mann, Nick Ryder, Melanie Subbiah, Jared Kaplan, Prafulla Dhariwal, Arvind Neelakantan, Pranav Shyam, Girish Sastry, Amanda Askell, Sandhini Agarwal, Ariel Herbert-Voss, Gretchen Krueger, Tom Henighan, Rewon Child, Aditya Ramesh, Daniel M. Ziegler, Jeffrey Wu, Clemens Winter, Christopher Hesse, Mark Chen, Eric Sigler, Mateusz Litwin, Scott Gray, Benjamin Chess, Jack Clark, Christopher Berner, Sam McCandlish, Alec Radford, Ilya Sutskever, and Dario Amodei. 2020.

[Language models are few-shot learners](http://arxiv.org/abs/2005.14165).

- Clem and Thomson (2021)
  Timothy Clem and Patrick Thomson. 2021.

Static analysis at github: An experience report.

_Queue_, 19(4):42–67.

- Iyer et al. (2018)
  Srinivasan Iyer, Ioannis Konstas, Alvin Cheung, and Luke Zettlemoyer. 2018.

Mapping language to code in programmatic context.

_arXiv preprint arXiv:1808.09588_.

- Latif et al. (2023)
  Afshan Latif, Farooque Azam, Muhammad Waseem Anwar, and Amina Zafar. 2023.

Comparison of leading language parsers–antlr, javacc, sablecc, tree-sitter, yacc, bison.

In _2023 13th International Conference on Software Technology and Engineering (ICSTE)_, pages 7–13. IEEE.

- Liao et al. (2023)
  Dianshu Liao, Shidong Pan, Qing Huang, Xiaoxue Ren, Zhenchang Xing, Huan Jin, and Qinying Li. 2023.

Context-aware code generation framework for code repositories: Local, global, and third-party library awareness.

_arXiv preprint arXiv:2312.05772_.

- Liu et al. (2023)
  Xiao Liu, Yanan Zheng, Zhengxiao Du, Ming Ding, Yujie Qian, Zhilin Yang, and Jie Tang. 2023.

Gpt understands, too.

_AI Open_.

- Papineni et al. (2002)
  Kishore Papineni, Salim Roukos, Todd Ward, and Wei-Jing Zhu. 2002.

[Bleu: a method for automatic evaluation of machine translation](https://doi.org/10.3115/1073083.1073135).

In _Proceedings of the 40th Annual Meeting on Association for Computational Linguistics_, ACL ’02, page 311–318, USA. Association for Computational Linguistics.

- Pawlik and Augsten (2015)
  Mateusz Pawlik and Nikolaus Augsten. 2015.

Efficient computation of the tree edit distance.

_ACM Transactions on Database Systems (TODS)_, 40(1):1–40.

- Pawlik and Augsten (2016)
  Mateusz Pawlik and Nikolaus Augsten. 2016.

Tree edit distance: Robust and memory-efficient.

_Information Systems_, 56:157–173.

- Song et al. (2023)
  Yewei Song, Saad Ezzini, Xunzhu Tang, Cedric Lothritz, Jacques Klein, Tegawendé Bissyandé, Andrey Boytsov, Ulrick Ble, and Anne Goujon. 2023.

Enhancing text-to-sql translation for financial system design.

_arXiv preprint arXiv:2312.14725_.

- Tang et al. (2024)
  Daniel Tang, Zhenghan Chen, Kisub Kim, Yewei Song, Haoye Tian, Saad Ezzini, Yongfeng Huang, and Jacques Klein Tegawende F Bissyande. 2024.

Collaborative agents for software engineering.

_arXiv preprint arXiv:2402.02172_.

- Tian et al. (2023)
  Haoye Tian, Weiqi Lu, Tsz On Li, Xunzhu Tang, Shing-Chi Cheung, Jacques Klein, and Tegawendé F Bissyandé. 2023.

Is chatgpt the ultimate programming assistant–how far is it?

_arXiv preprint arXiv:2304.11938_.

- Tomita (1991)
  Masaru Tomita. 1991.

_Generalized LR parsing_.

Springer Science & Business Media.

- Wang et al. (2023)
  Junjie Wang, Yuchao Huang, Chunyang Chen, Zhe Liu, Song Wang, and Qing Wang. 2023.

Software testing with large language model: Survey, landscape, and vision.

_arXiv preprint arXiv:2307.07221_.

- Yang et al. (2023)
  John Yang, Akshara Prabhakar, Karthik Narasimhan, and Shunyu Yao. 2023.

Intercode: Standardizing and benchmarking interactive coding with execution feedback.

_arXiv preprint arXiv:2306.14898_.

- Yin et al. (2018)
  Pengcheng Yin, Bowen Deng, Edgar Chen, Bogdan Vasilescu, and Graham Neubig. 2018.

Learning to mine aligned code and natural language pairs from stack overflow.

In _Proceedings of the 15th international conference on mining software repositories_, pages 476–486.

- Yu et al. (2024)
  Hao Yu, Bo Shen, Dezhi Ran, Jiaxin Zhang, Qi Zhang, Yuchi Ma, Guangtai Liang, Ying Li, Qianxiang Wang, and Tao Xie. 2024.

Codereval: A benchmark of pragmatic code generation with generative pre-trained models.

In _Proceedings of the 46th IEEE/ACM International Conference on Software Engineering_, pages 1–12.

- Yu et al. (2018)
  Tao Yu, Rui Zhang, Kai Yang, Michihiro Yasunaga, Dongxu Wang, Zifan Li, James Ma, Irene Li, Qingning Yao, Shanelle Roman, et al. 2018.

Spider: A large-scale human-labeled dataset for complex and cross-domain semantic parsing and text-to-sql task.

_arXiv preprint arXiv:1809.08887_.

## Appendix A Additional Experiment Details

### A.1 Parser Comparison

The ANTLR 3 3 3 https://www.antlr.org/ (ANother Tool for Language Recognition) tool, serving as a distinct AST parser compared to tree-sitter, demonstrated notable differences. Following our evaluation using identical settings for TSED metrics, as Figure [5](https://arxiv.org/html/2404.08817v1#A1.F5) shows, it became evident that the correlation with other metrics was inferior to the original solutions. This experiment underscores the crucial role of parser performance in the computation procedure, highlighting the significance of selecting an appropriate parser for accurate and reliable code similarity assessments.

![Refer to caption]() Figure 5: CoderEval Java Pearson Correlation Heatmap between evaluation-metrics/models/languages on TSED with ANTLR parser ### A.2 Other results

Due to space constraints, a subset of experimental data is provided in the appendix. A comprehensive evaluation of CoderEval and InterCoder is detailed in Table [5](https://arxiv.org/html/2404.08817v1#A1.T5), while specific original sample data from the MBXP dataset is presented in Table [6](https://arxiv.org/html/2404.08817v1#A1.T6).

CoderEval, designed for class-level code generation tasks, proves to be a challenging test. Utilizing Pass@10 data as a test sample, TSED demonstrates a robust correlation with semantic indicators in both Java and Python languages. Additionally, a noteworthy correlation is observed between TSED and GPT Similarity.

In the case of InterCoder, we confirm that TSED calculations extend to Bash scripts. Also, the correlation in Figure [6](https://arxiv.org/html/2404.08817v1#A1.F6) between TSED to semantic metrics is acceptable, the GPT score doesn’t have a good correlation to others. We also replicate the performance of the SPIDER dataset, noting differences from the original paper but not to a significant extent.

Despite the notably low semantic similarity between the MBXP built-in samples and the ground truth, a relatively high execution match is observed. We acknowledge this disparity and plan to address it through optimization in future research endeavors.

| Table 5: 4 Evaluation Metrics compared to Ground Truth on CoderEval(Java & Python) / InterCode(Bash) / SPIDER(SQL) | Languages   | Model  | TSED   | BLEU   | Jaccard Sim | GPT-4  | Execution |
| ------------------------------------------------------------------------------------------------------------------ | ----------- | ------ | ------ | ------ | ----------- | ------ | --------- |
| Java                                                                                                               | ChatGPT     | 0.4971 | 0.3655 | 0.3384 | 0.7392      | 0.3539 |
|                                                                                                                    | CodeGen     | 0.3616 | 0.2871 | 0.2506 | 0.6603      | 0.1391 |
|                                                                                                                    | PanGu       | 0.5029 | 0.3722 | 0.3849 | 0.6778      | 0.2543 |
| Python                                                                                                             | ChatGPT     | 0.2840 | 0.1285 | 0.1763 | 0.5883      | 0.2104 |
|                                                                                                                    | CodeGen     | 0.2703 | 0.1778 | 0.1821 | 0.5604      | 0.0948 |
|                                                                                                                    | PanGu       | 0.2829 | 0.0868 | 0.1567 | 0.5086      | 0.1183 |
| Shell                                                                                                              | GPT-4       | 0.5853 | 0.2816 | 0.3567 | 0.8511      | 0.4851 |
| starchat                                                                                                           | 0.4065      | 0.1594 | 0.2081 | 0.6740 | 0.2374      |        |
| vicuna                                                                                                             | 0.4755      | 0.1621 | 0.2295 | 0.7164 | 0.2451      |        |
| SQL                                                                                                                | ChatGPT-3.5 | 0.6824 | 0.3304 | 0.3710 | 0.9461      | 0.6482 |
| nsql-6B                                                                                                            | 0.8022      | 0.4493 | 0.4356 | 0.9265 | 0.5483      |        |
| RESDSQL                                                                                                            | 0.7422      | 0.2084 | 0.1868 | 0.9629 | 0.7756      |        |

| Table 6: 4 Evaluation Metrics compare to Ground Truth on 7 languages MBXP Dataset Samples | Languages | TSED   | BLEU   | Jaccard Sim | GPT-4 | Execution |
| ----------------------------------------------------------------------------------------- | --------- | ------ | ------ | ----------- | ----- | --------- |
| Java                                                                                      | 0.2218    | 0.1046 | 0.1960 | 0.4248      | 0.853 |
| Python                                                                                    | 0.1550    | 0.0255 | 0.1222 | 0.3396      | 0.822 |
| JavaScript                                                                                | 0.1870    | 0.0573 | 0.1685 | 0.4005      | 0.786 |
| Typescript                                                                                | 0.1186    | 0.0288 | 0.1260 | 0.4247      | 0.872 |
| Ruby                                                                                      | 0.2073    | 0.0235 | 0.1796 | 0.4830      | 0.589 |
| Kotlin                                                                                    | 0.1720    | 0.0336 | 0.1877 | 0.3976      | 0.637 |

![Refer to caption]() Figure 6: InterCode/SPIDER Pearson Correlation Heatmap between evaluation-metrics/models/languages
