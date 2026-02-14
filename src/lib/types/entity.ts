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
  defaultValue: number;
  description: string;
}

export interface EntitySchema {
  name: string;
  slug: string;
  icon: string;
  color: string;
  description: string;
  fields: EntityField[];
  spiderChart: SpiderAxis[];
}

export interface EntityInstance {
  slug: string;
  title: string;
  schemaSlug: string;
  fields: Record<string, string | number | boolean>;
  spiderValues: Record<string, number>;
  tags: string[];
  body: string;
  createdAt: string;
  updatedAt: string;
}

export interface EntitySummary {
  slug: string;
  title: string;
  schemaSlug: string;
  tags: string[];
}
