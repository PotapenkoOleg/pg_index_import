pub const SQL_SERVER_INDEX_EXTRACT_QUERY: &str = r#"
WITH base AS (SELECT t.object_id,
                     -- Pre-collate names
                     QUOTENAME(s.name) COLLATE DATABASE_DEFAULT AS q_schema,
                     QUOTENAME(t.name) COLLATE DATABASE_DEFAULT AS q_table
              FROM sys.tables t
                       JOIN sys.schemas s ON s.schema_id = t.schema_id
              WHERE s.name = @SchemaName
                AND t.name = @TableName),
     idx AS (SELECT i.object_id,
                    i.index_id,
                    i.name,
                    i.type,
                    i.type_desc,
                    i.is_unique,
                    i.is_primary_key,
                    i.is_unique_constraint,
                    i.has_filter,
                    -- Pre-collate filter text
                    i.filter_definition COLLATE DATABASE_DEFAULT AS filter_definition,
                    i.fill_factor,
                    i.is_padded,
                    i.ignore_dup_key,
                    i.allow_row_locks,
                    i.allow_page_locks,
                    i.is_disabled,
                    i.data_space_id
             FROM sys.indexes i
                      JOIN base b ON b.object_id = i.object_id
             WHERE i.index_id > 0),
     keycols AS (SELECT ic.object_id,
                        ic.index_id,
                        -- Make the aggregation a LOB and normalize collation; separator must be a literal
                        STRING_AGG(
                                CAST(
                                        (QUOTENAME(c.name)
                                            + CASE WHEN ic.is_descending_key = 1 THEN N' DESC' ELSE N' ASC' END
                                            ) AS NVARCHAR(MAX)
                                ) COLLATE DATABASE_DEFAULT,
                                N', '
                        ) WITHIN GROUP (ORDER BY ic.key_ordinal) COLLATE DATABASE_DEFAULT AS key_list
                 FROM sys.index_columns ic
                          JOIN sys.columns c
                               ON c.object_id = ic.object_id
                                   AND c.column_id = ic.column_id
                 WHERE ic.is_included_column = 0
                 GROUP BY ic.object_id, ic.index_id),
     inccols AS (SELECT ic.object_id,
                        ic.index_id,
                        STRING_AGG(
                                CAST(QUOTENAME(c.name) AS NVARCHAR(MAX)) COLLATE DATABASE_DEFAULT,
                                N', '
                        ) COLLATE DATABASE_DEFAULT AS include_list
                 FROM sys.index_columns ic
                          JOIN sys.columns c
                               ON c.object_id = ic.object_id
                                   AND c.column_id = ic.column_id
                 WHERE ic.is_included_column = 1
                 GROUP BY ic.object_id, ic.index_id),
     ds AS (SELECT ds.data_space_id,
                   ds.name COLLATE DATABASE_DEFAULT AS data_space_name,
                   ds.type                          AS data_space_type
            FROM sys.data_spaces ds),
-- Per-partition compression info
     part_comp AS (SELECT p.object_id,
                          p.index_id,
                          CASE
                              WHEN COUNT(DISTINCT p.data_compression_desc) = 1
                                  THEN MAX(p.data_compression_desc)
                              ELSE NULL END AS uniform_compression,
                          CASE
                              WHEN COUNT(DISTINCT p.data_compression_desc) > 1 THEN
                                  STRING_AGG(
                                          CAST(CONCAT(p.partition_number, N' = ', p.data_compression_desc) AS NVARCHAR(MAX)) COLLATE DATABASE_DEFAULT,
                                          N', '
                                  ) WITHIN GROUP (ORDER BY p.partition_number) COLLATE DATABASE_DEFAULT
                              END           AS partition_map
                   FROM sys.partitions p
                   GROUP BY p.object_id, p.index_id)
SELECT QUOTENAME(i.name),
    -- Force the whole expression to NVARCHAR(MAX) and normalize collation
    CAST((
        CASE
            WHEN i.is_primary_key = 1 THEN
                -- PKs via ALTER TABLE ... ADD CONSTRAINT
                CONCAT(
                        CAST(N'' AS NVARCHAR(MAX)),
                        N'ALTER TABLE ' COLLATE DATABASE_DEFAULT, b.q_schema, N'.' COLLATE DATABASE_DEFAULT, b.q_table,
                        N' ADD CONSTRAINT ' COLLATE DATABASE_DEFAULT, QUOTENAME(i.name) COLLATE DATABASE_DEFAULT,
                        N' PRIMARY KEY ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.type_desc = 'CLUSTERED' THEN N'CLUSTERED' ELSE N'NONCLUSTERED' END,
                        N' (' COLLATE DATABASE_DEFAULT, ISNULL(k.key_list, N'') COLLATE DATABASE_DEFAULT, N')',
                        CASE
                            WHEN i.has_filter = 1 THEN CONCAT(N' WHERE ' COLLATE DATABASE_DEFAULT, i.filter_definition)
                            ELSE N'' END,
                        CASE
                            WHEN d.data_space_type = 'FG' THEN CONCAT(N' ON ' COLLATE DATABASE_DEFAULT,
                                                                      QUOTENAME(d.data_space_name))
                            WHEN d.data_space_type = 'PS' THEN CONCAT(N' ON ' COLLATE DATABASE_DEFAULT,
                                                                      QUOTENAME(d.data_space_name), N'(',
                                                                      N'PARTITION_COLUMN_HERE', N')')
                            ELSE N''
                            END,
                        N' WITH (' COLLATE DATABASE_DEFAULT,
                        N'PAD_INDEX = ' COLLATE DATABASE_DEFAULT, CASE WHEN i.is_padded = 1 THEN N'ON' ELSE N'OFF' END,
                        N', FILLFACTOR = ' COLLATE DATABASE_DEFAULT, CASE
                                                                         WHEN i.fill_factor BETWEEN 1 AND 100
                                                                             THEN CONVERT(nvarchar(3), i.fill_factor)
                                                                         ELSE N'0' END,
                        N', IGNORE_DUP_KEY = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.ignore_dup_key = 1 THEN N'ON' ELSE N'OFF' END,
                        N', ALLOW_ROW_LOCKS = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.allow_row_locks = 1 THEN N'ON' ELSE N'OFF' END,
                        N', ALLOW_PAGE_LOCKS = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.allow_page_locks = 1 THEN N'ON' ELSE N'OFF' END,
                        N')',
                        CASE
                            WHEN pc.uniform_compression IS NOT NULL AND pc.uniform_compression <> 'NONE'
                                THEN CONCAT(N' WITH (DATA_COMPRESSION = ' COLLATE DATABASE_DEFAULT,
                                            pc.uniform_compression, N')')
                            WHEN pc.partition_map IS NOT NULL
                                THEN CONCAT(
                                    N' WITH (DATA_COMPRESSION = ROW | PAGE ON PARTITIONS (' COLLATE DATABASE_DEFAULT,
                                    pc.partition_map, N'))')
                            ELSE N''
                            END,
                        N';'
                )
            ELSE
                -- Regular indexes
                CONCAT(
                        CAST(N'' AS NVARCHAR(MAX)),
                        (CASE WHEN i.is_unique = 1 THEN N'CREATE UNIQUE ' ELSE N'CREATE ' END) COLLATE DATABASE_DEFAULT,
                        (CASE
                             WHEN i.type_desc = 'CLUSTERED' THEN N'CLUSTERED '
                             ELSE N'NONCLUSTERED ' END) COLLATE DATABASE_DEFAULT,
                        N'INDEX ' COLLATE DATABASE_DEFAULT, QUOTENAME(i.name) COLLATE DATABASE_DEFAULT,
                        N' ON ' COLLATE DATABASE_DEFAULT, b.q_schema, N'.' COLLATE DATABASE_DEFAULT, b.q_table,
                        N' (' COLLATE DATABASE_DEFAULT, ISNULL(k.key_list, N'') COLLATE DATABASE_DEFAULT, N')',
                        CASE
                            WHEN inc.include_list IS NOT NULL THEN CONCAT(N' INCLUDE (' COLLATE DATABASE_DEFAULT,
                                                                          inc.include_list, N')')
                            ELSE N'' END,
                        CASE
                            WHEN i.has_filter = 1 THEN CONCAT(N' WHERE ' COLLATE DATABASE_DEFAULT, i.filter_definition)
                            ELSE N'' END,
                        CASE
                            WHEN d.data_space_type = 'FG' THEN CONCAT(N' ON ' COLLATE DATABASE_DEFAULT,
                                                                      QUOTENAME(d.data_space_name))
                            WHEN d.data_space_type = 'PS' THEN CONCAT(N' ON ' COLLATE DATABASE_DEFAULT,
                                                                      QUOTENAME(d.data_space_name), N'(',
                                                                      N'PARTITION_COLUMN_HERE', N')')
                            ELSE N''
                            END,
                        N' WITH (' COLLATE DATABASE_DEFAULT,
                        N'PAD_INDEX = ' COLLATE DATABASE_DEFAULT, CASE WHEN i.is_padded = 1 THEN N'ON' ELSE N'OFF' END,
                        N', FILLFACTOR = ' COLLATE DATABASE_DEFAULT, CASE
                                                                         WHEN i.fill_factor BETWEEN 1 AND 100
                                                                             THEN CONVERT(nvarchar(3), i.fill_factor)
                                                                         ELSE N'0' END,
                        N', IGNORE_DUP_KEY = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.ignore_dup_key = 1 THEN N'ON' ELSE N'OFF' END,
                        N', ALLOW_ROW_LOCKS = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.allow_row_locks = 1 THEN N'ON' ELSE N'OFF' END,
                        N', ALLOW_PAGE_LOCKS = ' COLLATE DATABASE_DEFAULT,
                        CASE WHEN i.allow_page_locks = 1 THEN N'ON' ELSE N'OFF' END,
                        N')',
                        CASE
                            WHEN pc.uniform_compression IS NOT NULL AND pc.uniform_compression <> 'NONE'
                                THEN CONCAT(N' WITH (DATA_COMPRESSION = ' COLLATE DATABASE_DEFAULT,
                                            pc.uniform_compression, N')')
                            WHEN pc.partition_map IS NOT NULL
                                THEN CONCAT(
                                    N' WITH (DATA_COMPRESSION = ROW | PAGE ON PARTITIONS (' COLLATE DATABASE_DEFAULT,
                                    pc.partition_map, N'))')
                            ELSE N''
                            END,
                        N';',
                        CASE
                            WHEN i.is_disabled = 1 THEN CONCAT(
                                    N' -- NOTE: Index is currently DISABLED. To match state: ALTER INDEX ' COLLATE DATABASE_DEFAULT,
                                    QUOTENAME(i.name), N' ON ' COLLATE DATABASE_DEFAULT, b.q_schema,
                                    N'.' COLLATE DATABASE_DEFAULT, b.q_table, N' DISABLE;')
                            ELSE N'' END
                )
            END
        ) AS NVARCHAR(MAX)) COLLATE DATABASE_DEFAULT AS index_ddl
FROM idx i
         JOIN base b ON b.object_id = i.object_id
         LEFT JOIN keycols k ON k.object_id = i.object_id AND k.index_id = i.index_id
         LEFT JOIN inccols inc ON inc.object_id = i.object_id AND inc.index_id = i.index_id
         LEFT JOIN ds d ON d.data_space_id = i.data_space_id
         LEFT JOIN part_comp pc ON pc.object_id = i.object_id AND pc.index_id = i.index_id
ORDER BY i.is_primary_key DESC, i.is_unique DESC, i.name;
"#;