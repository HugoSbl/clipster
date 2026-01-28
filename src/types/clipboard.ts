/**
 * Content type for clipboard items
 * Matches Rust enum: crate::models::ContentType
 */
export type ContentType = 'text' | 'image' | 'files' | 'link' | 'audio' | 'documents';

/**
 * Clipboard item structure
 * Matches Rust struct: crate::models::ClipboardItem
 */
export interface ClipboardItem {
  id: string;
  content_type: ContentType;
  content_text: string | null;
  thumbnail_base64: string | null;
  image_path: string | null;
  source_app: string | null;
  source_app_icon: string | null;
  created_at: string;
  pinboard_id: string | null;
  is_favorite: boolean;
}

/**
 * Pinboard structure
 * Matches Rust struct: crate::models::Pinboard
 */
export interface Pinboard {
  id: string;
  name: string;
  icon: string | null;
  position: number;
  created_at: string;
}

/**
 * Payload for clipboard-changed event
 * Matches Rust struct: ClipboardChangedPayload
 */
export interface ClipboardChangedPayload {
  item: ClipboardItem;
  /** If this item replaced an existing one (move to top), contains the old item's ID */
  replaced_item_id?: string;
}

/**
 * Payload for clipboard-item-thumbnail-updated event
 * Matches Rust struct: ThumbnailUpdatedPayload
 */
export interface ThumbnailUpdatedPayload {
  id: string;
  thumbnail_base64: string;
}
