-- Estela: migración inicial (no modificar después de publicar).
CREATE SCHEMA IF NOT EXISTS library AUTHORIZATION estela_app;
SET search_path TO library;
CREATE TYPE recording_status AS ENUM ('recording','ok','recovered','missing','corrupt');
CREATE TYPE platform_kind AS ENUM ('windows','linux','macos');
CREATE TYPE video_codec AS ENUM ('h264','hevc','av1');
CREATE TYPE export_container AS ENUM ('mp4','webm','gif');
CREATE TYPE export_status AS ENUM ('queued','running','done','failed','cancelled');
CREATE TYPE decode_backend AS ENUM ('hardware','software');
CREATE TABLE library_roots (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), path text NOT NULL UNIQUE,
 is_default boolean NOT NULL DEFAULT false, added_at timestamptz NOT NULL DEFAULT now(), last_scan_at timestamptz
);
CREATE UNIQUE INDEX library_roots_one_default ON library_roots(is_default) WHERE is_default;
CREATE TABLE recordings (
 id uuid PRIMARY KEY, root_id uuid REFERENCES library_roots(id) ON DELETE SET NULL,
 path text NOT NULL UNIQUE, file_size bigint NOT NULL CHECK(file_size>=0), file_mtime timestamptz NOT NULL,
 title text NOT NULL, recorded_at timestamptz NOT NULL, duration_ns bigint NOT NULL CHECK(duration_ns>=0),
 width integer NOT NULL CHECK(width>0), height integer NOT NULL CHECK(height>0), video_codec video_codec NOT NULL,
 has_audio boolean NOT NULL, platform platform_kind NOT NULL, app_version text NOT NULL,
 format_major smallint NOT NULL, format_minor smallint NOT NULL, status recording_status NOT NULL DEFAULT 'ok',
 poster_path text, indexed_at timestamptz NOT NULL DEFAULT now(),
 search tsvector GENERATED ALWAYS AS (to_tsvector('spanish',title)) STORED
);
CREATE INDEX recordings_recorded_at ON recordings(recorded_at DESC,id DESC);
CREATE INDEX recordings_root ON recordings(root_id);
CREATE INDEX recordings_not_ok ON recordings(status) WHERE status<>'ok';
CREATE INDEX recordings_search ON recordings USING gin(search);
CREATE TABLE tags (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), name text NOT NULL CHECK(length(btrim(name))>0), color text CHECK(color ~ '^#[0-9A-Fa-f]{6}$'));
CREATE UNIQUE INDEX tags_name_ci ON tags(lower(name));
CREATE TABLE recording_tags (
 recording_id uuid NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
 tag_id uuid NOT NULL REFERENCES tags(id) ON DELETE CASCADE, PRIMARY KEY(recording_id,tag_id)
);
CREATE INDEX recording_tags_tag ON recording_tags(tag_id);
CREATE TABLE collections(id uuid PRIMARY KEY DEFAULT gen_random_uuid(),name text NOT NULL CHECK(length(btrim(name))>0),created_at timestamptz NOT NULL DEFAULT now());
CREATE TABLE collection_items (
 collection_id uuid NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
 recording_id uuid NOT NULL REFERENCES recordings(id) ON DELETE CASCADE,
 position integer NOT NULL CHECK(position>=0), PRIMARY KEY(collection_id,recording_id),
 CONSTRAINT collection_items_order UNIQUE(collection_id,position) DEFERRABLE INITIALLY DEFERRED
);
CREATE INDEX collection_items_recording ON collection_items(recording_id);
CREATE TABLE exports (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),recording_id uuid REFERENCES recordings(id) ON DELETE SET NULL,
 status export_status NOT NULL DEFAULT 'queued',container export_container NOT NULL,video_codec video_codec,
 width integer NOT NULL CHECK(width>0),height integer NOT NULL CHECK(height>0),fps numeric(6,3) NOT NULL CHECK(fps>0),
 bitrate_kbps integer CHECK(bitrate_kbps>0),output_path text NOT NULL,encoder text,decode_backend decode_backend,
 queued_at timestamptz NOT NULL DEFAULT now(),started_at timestamptz,finished_at timestamptz,error text,
 CHECK(container='gif' OR video_codec IS NOT NULL),CHECK(finished_at IS NULL OR started_at IS NOT NULL),CHECK(status<>'failed' OR error IS NOT NULL)
);
CREATE INDEX exports_recording ON exports(recording_id,queued_at DESC);
CREATE INDEX exports_active ON exports(queued_at) WHERE status IN ('queued','running');
CREATE TABLE index_runs (
 id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,root_id uuid REFERENCES library_roots(id) ON DELETE CASCADE,
 started_at timestamptz NOT NULL DEFAULT now(),finished_at timestamptz,found integer NOT NULL DEFAULT 0,
 added integer NOT NULL DEFAULT 0,updated integer NOT NULL DEFAULT 0,missing integer NOT NULL DEFAULT 0,errors jsonb NOT NULL DEFAULT '[]'::jsonb
);
CREATE INDEX index_runs_root ON index_runs(root_id,started_at DESC);
CREATE TABLE machine_capabilities (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),os text NOT NULL,gpu_name text NOT NULL,gpu_vendor text NOT NULL,
 driver_version text NOT NULL,decode jsonb NOT NULL,encode jsonb NOT NULL,probed_at timestamptz NOT NULL DEFAULT now(),UNIQUE(os,gpu_name,driver_version)
);
CREATE TABLE settings(key text PRIMARY KEY,value jsonb NOT NULL,updated_at timestamptz NOT NULL DEFAULT now());
