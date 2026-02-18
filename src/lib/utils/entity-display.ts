import type { ComponentType } from 'svelte';
import { Users, MapPin, Package, Lightbulb, File } from 'lucide-svelte';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type IconComponent = ComponentType<any>;

/** Default icon mapping for known entity types */
export const ENTITY_ICONS: Record<string, IconComponent> = {
  character: Users,
  place: MapPin,
  item: Package,
  idea: Lightbulb,
};

/** Default color mapping for known entity types */
export const ENTITY_COLORS: Record<string, string> = {
  character: '#7c4dbd',
  place: '#2e8b57',
  item: '#c28a1e',
  idea: '#3a7bd5',
};

/** Get the icon component for an entity type, with File as fallback */
export function getEntityIcon(entityType: string): IconComponent {
  return ENTITY_ICONS[entityType] ?? File;
}

/** Get the color for an entity type, or undefined for unknown types */
export function getEntityColor(entityType: string): string | undefined {
  return ENTITY_COLORS[entityType];
}
