/*  ___ _  ___ _ _                                                            *\
** / __| |/ (_) | |       The SKilL Generator                                 **
** \__ \ ' <| | | |__     (c) 2013 University of Stuttgart                    **
** |___/_|\_\_|_|____|    see LICENSE                                         **
\*                                                                            */
package de.ust.skill.generator.scala.internal.streams

import java.io.PrintWriter
import de.ust.skill.generator.scala.GeneralOutputMaker

trait OutBufferMaker extends GeneralOutputMaker {
  abstract override def make {
    super.make
    val out = open("internal/streams/OutBuffer.java")
    //package
    out.write(s"""package ${packagePrefix}internal.streams;

/**
 * Store data as a non-empty single linked list.
 * 
 * @note we do not want to use an array here, because we would run into jvm
 *       limits before running out of memory - sadly.
 * 
 * @author Timm Felden
 */
final public class OutBuffer extends OutStream {
	static abstract class Data {
		public Data next = null;
	}

	static final class NoData extends Data {
		// is used for roots; can appear everywhere!
	}

	static final class ByteData extends Data {
		public final byte data;

		ByteData(byte data) {
			this.data = data;
		}

		ByteData(byte data, Data tail) {
			tail.next = this;
			this.data = data;
		}
	}

	static final class BulkData extends Data {
		public final byte[] data;
		/**
		 * Number of used bytes in data. This is required for v64 caching.
		 */
		public int used;

		BulkData(Data tail) {
			data = new byte[8 * 1024];
			used = 0;
			tail.next = this;
		}

		BulkData(byte[] data) {
			this.data = data;
			this.used = data.length;
		}

		BulkData(byte[] data, Data tail) {
			tail.next = this;
			this.data = data;
			this.used = data.length;
		}
	}

	final Data head;
	private Data tail;
	private long size;

	public long size() {
		return size;
	}

	public OutBuffer() {
		head = new NoData();
		tail = head;
		size = 0;
	}

	public OutBuffer(byte data) {
		head = new ByteData(data);
		tail = head;
		size = 1;
	}

	public OutBuffer(byte[] data) {
		head = new BulkData(data);
		tail = head;
		size = data.length;
	}

	@Override
	public void put(byte data) {
		tail = new ByteData(data, tail);
		size++;
	}

	@Override
	public void put(byte[] data) {
		tail = new BulkData(data, tail);
		size += data.length;
	}

	@Override
	public void putAll(OutBuffer stream) throws Exception {
		tail.next = stream.head;
		tail = stream.tail;
		size += stream.size;
	}

	@Override
	public void close() throws Exception {
		// irrelevant
	}

	@Override
	public void v64(long v) throws Exception {
		if (!(tail instanceof BulkData) || ((BulkData) tail).data.length - ((BulkData) tail).used < 9)
			tail = new BulkData(tail);

		BulkData out = (BulkData) tail;
		byte[] data = out.data;
		int off = out.used;

		if (0L == (v & 0xFFFFFFFFFFFFFF80L)) {
			data[off++] = (byte) v;
		} else if (0L == (v & 0xFFFFFFFFFFFFC000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (v >> 7);
		} else if (0L == (v & 0xFFFFFFFFFFE00000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (v >> 14);
		} else if (0L == (v & 0xFFFFFFFFF0000000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (v >> 21);
		} else if (0L == (v & 0xFFFFFFF800000000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (0x80L | v >> 21);
			data[off++] = (byte) (v >> 28);
		} else if (0L == (v & 0xFFFFFC0000000000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (0x80L | v >> 21);
			data[off++] = (byte) (0x80L | v >> 28);
			data[off++] = (byte) (v >> 35);
		} else if (0L == (v & 0xFFFE000000000000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (0x80L | v >> 21);
			data[off++] = (byte) (0x80L | v >> 28);
			data[off++] = (byte) (0x80L | v >> 35);
			data[off++] = (byte) (v >> 42);
		} else if (0L == (v & 0xFF00000000000000L)) {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (0x80L | v >> 21);
			data[off++] = (byte) (0x80L | v >> 28);
			data[off++] = (byte) (0x80L | v >> 35);
			data[off++] = (byte) (0x80L | v >> 42);
			data[off++] = (byte) (v >> 49);
		} else {
			data[off++] = (byte) (0x80L | v);
			data[off++] = (byte) (0x80L | v >> 7);
			data[off++] = (byte) (0x80L | v >> 14);
			data[off++] = (byte) (0x80L | v >> 21);
			data[off++] = (byte) (0x80L | v >> 28);
			data[off++] = (byte) (0x80L | v >> 35);
			data[off++] = (byte) (0x80L | v >> 42);
			data[off++] = (byte) (0x80L | v >> 49);
			data[off++] = (byte) (v >> 56);
		}
		size += off - out.used;
		out.used = off;
	}
}
""")

    //class prefix
    out.close()
  }
}