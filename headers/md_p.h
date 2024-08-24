// trimmed md_p.h to '#define' as everything tagged __u32 crashes bindgen
// and i don't want to figure out why
//
#define MD_RESERVED_BYTES		(64 * 1024)
#define MD_RESERVED_SECTORS		(MD_RESERVED_BYTES / 512)
#define MD_RESERVED_BLOCKS		(MD_RESERVED_BYTES / BLOCK_SIZE)
#define MD_NEW_SIZE_SECTORS(x)		((x & ~(MD_RESERVED_SECTORS - 1)) - MD_RESERVED_SECTORS)
#define MD_NEW_SIZE_BLOCKS(x)		((x & ~(MD_RESERVED_BLOCKS - 1)) - MD_RESERVED_BLOCKS)
#define MD_SB_BYTES			4096
#define MD_SB_WORDS			(MD_SB_BYTES / 4)
#define MD_SB_BLOCKS			(MD_SB_BYTES / BLOCK_SIZE)
#define MD_SB_SECTORS			(MD_SB_BYTES / 512)
#define	MD_SB_GENERIC_OFFSET		0
#define MD_SB_PERSONALITY_OFFSET	64
#define MD_SB_DISKS_OFFSET		128
#define MD_SB_DESCRIPTOR_OFFSET		992
#define MD_SB_GENERIC_CONSTANT_WORDS	32
#define MD_SB_GENERIC_STATE_WORDS	32
#define MD_SB_GENERIC_WORDS		(MD_SB_GENERIC_CONSTANT_WORDS + MD_SB_GENERIC_STATE_WORDS)
#define MD_SB_PERSONALITY_WORDS		64
#define MD_SB_DESCRIPTOR_WORDS		32
#define MD_SB_DISKS			27
#define MD_SB_DISKS_WORDS		(MD_SB_DISKS*MD_SB_DESCRIPTOR_WORDS)
#define MD_SB_RESERVED_WORDS		(1024 - MD_SB_GENERIC_WORDS - MD_SB_PERSONALITY_WORDS - MD_SB_DISKS_WORDS - MD_SB_DESCRIPTOR_WORDS)
#define MD_SB_EQUAL_WORDS		(MD_SB_GENERIC_WORDS + MD_SB_PERSONALITY_WORDS + MD_SB_DISKS_WORDS)
#define MD_DISK_FAULTY		0 /* disk is faulty / operational */
#define MD_DISK_ACTIVE		1 /* disk is running but may not be in sync */
#define MD_DISK_SYNC		2 /* disk is in sync with the raid set */
#define MD_DISK_REMOVED		3 /* disk is in sync with the raid set */
#define MD_DISK_CLUSTER_ADD	4 /* Initiate a disk add across the cluster
#define MD_DISK_CANDIDATE	5 /* disk is added as spare (local) until confirmed
#define	MD_DISK_WRITEMOSTLY	9 /* disk is "write-mostly" is RAID1 config.
#define	MD_DISK_FAILFAST	10 /* Fewer retries, more failures */
#define MD_DISK_REPLACEMENT	17
#define MD_DISK_JOURNAL		18 /* disk is used as the write journal in RAID-5/6 */
#define MD_DISK_ROLE_SPARE	0xffff
#define MD_DISK_ROLE_FAULTY	0xfffe
#define MD_DISK_ROLE_JOURNAL	0xfffd
#define MD_DISK_ROLE_MAX	0xff00 /* max value of regular disk role */
#define MD_SB_MAGIC		0xa92b4efc
#define MD_SB_CLEAN		0
#define MD_SB_ERRORS		1
#define MD_SB_BBM_ERRORS	2
#define MD_SB_BLOCK_CONTAINER_RESHAPE 3 /* block container wide reshapes */
#define MD_SB_BLOCK_VOLUME	4 /* block activation of array, other arrays
#define MD_SB_CLUSTERED		5 /* MD is clustered  */
#define	MD_SB_BITMAP_PRESENT	8 /* bitmap may be present nearby */
#define R5LOG_VERSION 0x1
#define R5LOG_MAGIC 0x6433c509
#define PPL_HEADER_SIZE 4096
#define PPL_HDR_RESERVED 512
#define PPL_HDR_ENTRY_SPACE \
#define PPL_HDR_MAX_ENTRIES \
