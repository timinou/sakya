export type FieldType = 'short_text' | 'long_text' | 'number' | 'select' | 'date' | 'boolean';

export interface EntityField {
  name: string;
  label: string;
  fieldType: FieldType;
  placeholder?: string;
  description?: string;
  required?: boolean;
  options?: string[]; // for select type
  min?: number; // for number type
  max?: number; // for number type
  defaultValue?: string | number | boolean;
}

export interface SpiderAxis {
  name: string;
  min: number;
  max: number;
  default: number;
  description?: string;
}

export interface EntitySchema {
  name: string;
  entityType: string;
  icon?: string;
  color?: string;
  description?: string;
  fields: EntityField[];
  spiderAxes: SpiderAxis[];
}

export interface SchemaSummary {
  name: string;
  entityType: string;
  fieldCount: number;
  axisCount: number;
}

export interface EntityInstance {
  title: string;
  slug: string;
  schemaSlug: string;
  tags: string[];
  spiderValues: Record<string, number>;
  fields: Record<string, unknown>;
  body: string;
}

export interface EntitySummary {
  title: string;
  slug: string;
  schemaType: string;
  tags: string[];
}
