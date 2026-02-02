export interface AttributeStats {
    name: string;
    count: number;
    value_counts: Record<string, number>;
}

export interface TagStats {
    name: string;
    count: number;
    attributes: Record<string, AttributeStats>;
}

export interface AnalysisResult {
    tags: Record<string, TagStats>;
    files_analyzed: number;
    max_depth: number;
}
