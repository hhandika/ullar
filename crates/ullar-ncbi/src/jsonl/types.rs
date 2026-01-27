use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssemblyInfo {
    pub assembly_level: String,
    pub assembly_name: String,
    pub assembly_type: String,
    pub submitter: String,
    pub refseq_category: String,
    pub bioproject_lineage: Vec<BioprojectLineage>,
    pub sequencing_tech: String,
    pub blast_url: String,
    pub biosample: Biosample,
    pub comments: String,
    pub assembly_status: String,
    pub paired_assembly: PairedAssembly,
    pub bioproject_accession: String,
    pub assembly_method: String,
    pub release_date: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BioprojectLineage {
    pub bioprojects: Vec<Bioproject>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bioproject {
    pub accession: String,
    pub title: String,
    // Optional because it appears in some objects but not others in the lineage
    #[serde(default)]
    pub parent_accessions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Biosample {
    pub accession: String,
    pub last_updated: String,
    pub publication_date: String,
    pub submission_date: String,
    pub sample_ids: Vec<SampleId>,
    pub description: BiosampleDescription,
    pub owner: Owner,
    pub models: Vec<String>,
    pub package: String,
    pub attributes: Vec<Attribute>,
    pub status: BiosampleStatus,
    pub dev_stage: String,
    pub isolate: String,
    pub sex: String,
    pub tissue: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleId {
    pub label: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosampleDescription {
    pub title: String,
    pub organism: BiosampleOrganism,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosampleOrganism {
    pub tax_id: i64,
    pub organism_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub name: String,
    // Keeping as Value or generic JSON to be safe, as "contacts" was [{}] in sample
    pub contacts: Vec<serde_json::Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosampleStatus {
    pub status: String,
    pub when: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedAssembly {
    pub accession: String,
    pub status: String,
    pub only_refseq: String,
    pub refseq_genbank_are_different: bool,
    pub differences: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssemblyStats {
    pub total_number_of_chromosomes: i64,
    pub total_sequence_length: String, // String in JSON "2382770976"
    pub total_ungapped_length: String,
    pub number_of_contigs: i64,
    pub contig_n50: i64,
    pub contig_l50: i64,
    pub number_of_scaffolds: i64,
    pub scaffold_n50: i64,
    pub scaffold_l50: i64,
    pub number_of_component_sequences: i64,
    pub gc_count: String,
    pub gc_percent: f64,
    pub genome_coverage: String,
    pub number_of_organelles: i64,
    pub atgc_count: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganelleInfo {
    pub description: String,
    pub total_seq_length: String,
    pub submitter: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationInfo {
    pub name: String,
    pub provider: String,
    pub release_date: String,
    pub report_url: String,
    pub stats: AnnotationStats,
    pub busco: Busco,
    pub method: String,
    pub pipeline: String,
    pub software_version: String,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationStats {
    pub gene_counts: GeneCounts,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneCounts {
    pub total: i64,
    pub protein_coding: i64,
    pub non_coding: i64,
    pub pseudogene: i64,
    pub other: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Busco {
    pub busco_lineage: String,
    pub busco_ver: String,
    pub complete: f64,
    pub single_copy: f64,
    pub duplicated: f64,
    pub fragmented: f64,
    pub missing: f64,
    pub total_count: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WgsInfo {
    pub wgs_project_accession: String,
    pub master_wgs_url: String,
    pub wgs_contigs_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalSubmitter {
    pub genbank_accession: String,
    pub refseq_accession: String,
    pub chr_name: String,
    pub molecule_type: String,
    pub submitter: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organism {
    pub tax_id: i64,
    pub common_name: String,
    pub organism_name: String,
    pub infraspecific_names: InfraspecificNames,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfraspecificNames {
    pub isolate: String,
    pub sex: String,
}
