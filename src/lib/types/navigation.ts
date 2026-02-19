export type NavigationTarget =
  | { type: 'chapter'; slug: string }
  | { type: 'note'; slug: string }
  | { type: 'entity'; schemaType: string; slug: string }
  | { type: 'schema'; entityType: string }
  | { type: 'schema'; isNew: true }
  | { type: 'stats' };
