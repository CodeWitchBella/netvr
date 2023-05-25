using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Snapshot: IEquatable<Snapshot>, ICloneable {
        public Serde.ValueArray<Pose> objects;

        public Snapshot(Serde.ValueArray<Pose> _objects) {
            if (_objects == null) throw new ArgumentNullException(nameof(_objects));
            objects = _objects;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            TraitHelpers.serialize_vector_Pose(objects, serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static Snapshot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Snapshot obj = new Snapshot(
            	TraitHelpers.deserialize_vector_Pose(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Snapshot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Snapshot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Snapshot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Snapshot other && Equals(other);

        public static bool operator ==(Snapshot left, Snapshot right) => Equals(left, right);

        public static bool operator !=(Snapshot left, Snapshot right) => !Equals(left, right);

        public bool Equals(Snapshot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!objects.Equals(other.objects)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + objects.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Snapshot Clone() => (Snapshot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
